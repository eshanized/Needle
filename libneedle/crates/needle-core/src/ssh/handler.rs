// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::tunnel::manager::TunnelManager;
use async_trait::async_trait;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Handles one SSH client connection. Each connecting client gets its own
/// SshSession instance which lives for the duration of that connection.
///
/// The flow goes:
/// 1. Client connects and authenticates (we accept all keys for now,
///    API key validation will come later)
/// 2. Client requests a tcpip-forward for some port
/// 3. We create a tunnel in the TunnelManager, which gives us a local
///    TCP listener on 127.0.0.1
/// 4. When HTTP traffic comes in for that tunnel's subdomain, it gets
///    proxied to the local listener, and we forward the bytes back
///    and forth through the SSH channel
pub struct SshSession {
    tunnel_manager: Arc<RwLock<TunnelManager>>,
    client_ip: String,
    user_id: Uuid,
    allocated_subdomains: Vec<String>,
    channels: HashMap<ChannelId, String>,
}

impl SshSession {
    pub fn new(
        tunnel_manager: Arc<RwLock<TunnelManager>>,
        client_ip: String,
    ) -> Self {
        Self {
            tunnel_manager,
            client_ip,
            user_id: Uuid::new_v4(),
            allocated_subdomains: Vec::new(),
            channels: HashMap::new(),
        }
    }

    /// Sends a text message back to the client through their SSH channel.
    /// We use this to communicate tunnel URLs, errors, and status info
    /// since the client might be using a plain ssh command without our CLI.
    async fn send_message(session: &mut Session, channel: ChannelId, msg: &str) {
        session.data(channel, msg.as_bytes().into()).unwrap_or(());
    }
}

impl Drop for SshSession {
    fn drop(&mut self) {
        let manager = self.tunnel_manager.clone();
        let subdomains = self.allocated_subdomains.clone();

        tokio::spawn(async move {
            let mut mgr = manager.write().await;
            for sub in subdomains {
                if let Err(e) = mgr.remove(&sub).await {
                    error!(subdomain = %sub, error = %e, "failed to clean up tunnel on disconnect");
                }
            }
        });
    }
}

#[async_trait]
impl Handler for SshSession {
    type Error = russh::Error;

    /// We accept all public keys for now. In production this would
    /// validate against stored user keys or require the username to
    /// contain a valid API token.
    async fn auth_publickey(
        &mut self,
        user: &str,
        _public_key: &russh_keys::key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        info!(user = %user, ip = %self.client_ip, "ssh auth attempt");
        Ok(Auth::Accept)
    }

    async fn auth_none(&mut self, user: &str) -> Result<Auth, Self::Error> {
        debug!(user = %user, "auth_none, accepting for dev");
        Ok(Auth::Accept)
    }

    /// Called when the client opens a new session channel. We just accept
    /// it and wait for further requests on that channel.
    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        debug!(channel = %channel.id(), "session channel opened");
        Ok(true)
    }

    /// Handles the tcpip-forward request, which is how SSH reverse tunnels
    /// work. The client says "please forward traffic for port X to me" and
    /// we respond by creating a tunnel with a unique subdomain.
    async fn tcpip_forward(
        &mut self,
        address: &str,
        port: &mut u32,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        info!(
            address = %address,
            port = %port,
            ip = %self.client_ip,
            "tcpip-forward requested"
        );

        let mut manager = self.tunnel_manager.write().await;
        match manager.create(&self.client_ip, self.user_id).await {
            Ok(tunnel) => {
                let subdomain = tunnel.subdomain.clone();
                let bind_port = tunnel.bind_addr.port();

                self.allocated_subdomains.push(subdomain.clone());

                *port = bind_port as u32;

                info!(
                    subdomain = %subdomain,
                    bind_port = %bind_port,
                    "tunnel allocated for ssh client"
                );

                Ok(true)
            }
            Err(e) => {
                warn!(error = %e, "failed to create tunnel for ssh client");
                Ok(false)
            }
        }
    }

    /// Cleanup when the client cancels a forward or disconnects.
    async fn cancel_tcpip_forward(
        &mut self,
        _address: &str,
        _port: u32,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        debug!("tcpip-forward cancelled");
        Ok(true)
    }
}
