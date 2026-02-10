// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::metrics;
use crate::tunnel::manager::TunnelManager;
use async_trait::async_trait;
use needle_common::error::NeedleError;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Reserved/privileged ports that should not be allowed for SSH tunnels
const RESERVED_PORTS: &[u16] = &[22, 80, 443];
const MIN_ALLOWED_PORT: u16 = 1024;

/// Handles one SSH client connection. Each connecting client gets its own
/// SshSession instance which lives for the duration of that connection.
///
/// The flow goes:
/// 1. Client connects and authenticates via API key in username
///    Format: user_<API_KEY> where API_KEY is a 64-char hex string
/// 2. We validate the API key against the database
/// 3. Client requests a tcpip-forward for some port
/// 4. We validate the port is allowed (>= 1024, not reserved)
/// 5. We create a tunnel in the TunnelManager, which gives us a local
///    TCP listener on 127.0.0.1
/// 6. When HTTP traffic comes in for that tunnel's subdomain, it gets
///    proxied to the local listener, and we forward the bytes back
///    and forth through the SSH channel
pub struct SshSession {
    tunnel_manager: Arc<RwLock<TunnelManager>>,
    client_ip: String,
    user_id: Option<Uuid>,
    allocated_subdomains: Vec<String>,
    channels: HashMap<ChannelId, String>,
}

impl SshSession {
    pub fn new(tunnel_manager: Arc<RwLock<TunnelManager>>, client_ip: String) -> Self {
        Self {
            tunnel_manager,
            client_ip,
            user_id: None,
            allocated_subdomains: Vec::new(),
            channels: HashMap::new(),
        }
    }

    /// Sends a text message back to the client through their SSH channel.
    /// We use this to communicate tunnel URLs, errors, and status info
    /// since the client might be using a plain ssh command without our CLI.
    async fn send_message(session: &mut Session, channel: ChannelId, msg: &str) {
        session.data(channel, msg.as_bytes().to_vec().into());
    }

    /// Validates the API key extracted from the username.
    /// Returns the user_id if valid, None otherwise.
    async fn validate_api_key(&self, username: &str) -> Option<Uuid> {
        use sha2::{Digest, Sha256};

        // Extract API key from username (format: user_<KEY>)
        let api_key = username.strip_prefix("user_")?;

        // Validate key format (64 hex chars)
        if api_key.len() != 64 || !api_key.chars().all(|c| c.is_ascii_hexdigit()) {
            warn!("invalid api key format in username");
            return None;
        }

        // Hash the key
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        let key_hash = hex::encode(hasher.finalize());

        // Query database
        let db = {
            let mgr = self.tunnel_manager.read().await;
            mgr.db_client().clone()
        };

        match needle_db::queries::api_keys::find_by_hash(&db, &key_hash).await {
            Ok(Some(api_key_record)) => {
                info!("valid api key found for user {}", api_key_record.user_id);
                Some(api_key_record.user_id)
            }
            Ok(None) => {
                warn!("api key not found in database");
                None
            }
            Err(e) => {
                error!(error = %e, "database error during api key validation");
                None
            }
        }
    }

    /// Validates that the requested port is allowed for SSH tunnels.
    /// Rejects ports < 1024 and reserved ports (22, 80, 443).
    fn validate_port(port: u32) -> Result<u16, NeedleError> {
        if port > u16::MAX as u32 {
            return Err(NeedleError::InvalidPort {
                port: u16::MAX,
                min: MIN_ALLOWED_PORT,
            });
        }

        let port = port as u16;

        if port < MIN_ALLOWED_PORT {
            warn!(port = %port, "ssh port below minimum allowed");
            return Err(NeedleError::InvalidPort {
                port,
                min: MIN_ALLOWED_PORT,
            });
        }

        if RESERVED_PORTS.contains(&port) {
            warn!(port = %port, "ssh port is reserved");
            return Err(NeedleError::InvalidPort {
                port,
                min: MIN_ALLOWED_PORT,
            });
        }

        Ok(port)
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

    /// Validates API key from username field.
    /// Expected format: user_<API_KEY> where API_KEY is a 64-char hex string.
    /// The key is hashed with SHA-256 and validated against the database.
    async fn auth_publickey(
        &mut self,
        user: &str,
        _public_key: &russh_keys::key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        info!(user = %user, ip = %self.client_ip, "ssh auth attempt");

        if let Some(user_id) = self.validate_api_key(user).await {
            self.user_id = Some(user_id);
            info!(user_id = %user_id, "ssh authentication successful");
            Ok(Auth::Accept)
        } else {
            warn!(user = %user, ip = %self.client_ip, "ssh authentication failed");
            metrics::auth_failure("ssh", "invalid_key");
            Ok(Auth::Reject {
                proceed_with_methods: Some(russh::MethodSet::PUBLICKEY),
            })
        }
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
    ///
    /// Now includes port validation to prevent abuse of privileged ports.
    async fn tcpip_forward(
        &mut self,
        address: &str,
        port: &mut u32,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        // Ensure user is authenticated
        let user_id = match self.user_id {
            Some(id) => id,
            None => {
                warn!(ip = %self.client_ip, "tcpip-forward requested without authentication");
                return Ok(false);
            }
        };

        info!(
            address = %address,
            port = %port,
            ip = %self.client_ip,
            user_id = %user_id,
            "tcpip-forward requested"
        );

        // Validate port is allowed
        match Self::validate_port(*port) {
            Ok(_) => {} // Port is valid, continue
            Err(e) => {
                warn!(error = %e, requested_port = %port, "invalid port rejected");
                metrics::error_occurred("ssh_invalid_port");
                return Ok(false);
            }
        }

        let mut manager = self.tunnel_manager.write().await;
        match manager
            .create(
                &self.client_ip,
                user_id,
                None,         // No custom subdomain for SSH tunnels
                *port as i32, // Use requested port
                "http",       // Default protocol
                false,        // SSH tunnels are not persistent
            )
            .await
        {
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
