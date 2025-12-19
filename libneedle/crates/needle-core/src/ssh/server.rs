// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::ssh::handler::SshSession;
use crate::tunnel::manager::TunnelManager;
use russh::server::Config;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info};

const SSH_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const SSH_WINDOW_SIZE: u32 = 2_097_152;
const SSH_MAX_PACKET_SIZE: u32 = 32_768;

/// Runs the SSH server that accepts incoming reverse tunnel connections.
///
/// Clients connect with `ssh -R 80:localhost:3000 needle.example.com`
/// and we allocate a subdomain for them. This function blocks forever,
/// accepting connections in a loop and spawning a task per client.
pub async fn run(
    addr: &str,
    host_key: russh_keys::key::KeyPair,
    tunnel_manager: Arc<RwLock<TunnelManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config {
        auth_rejection_time: SSH_HANDSHAKE_TIMEOUT,
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        keys: vec![host_key],
        window_size: SSH_WINDOW_SIZE,
        maximum_packet_size: SSH_MAX_PACKET_SIZE,
        ..Default::default()
    });

    let listener = TcpListener::bind(addr).await?;
    info!(addr = %addr, "ssh server listening");

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let config = config.clone();
                let tm = tunnel_manager.clone();
                let client_ip = peer_addr.ip().to_string();

                tokio::spawn(async move {
                    let session = SshSession::new(tm, client_ip.clone());

                    let result = russh::server::run_stream(config, stream, session).await;

                    match result {
                        Ok(_) => info!(ip = %client_ip, "ssh session ended cleanly"),
                        Err(e) => error!(ip = %client_ip, error = %e, "ssh session error"),
                    }
                });
            }
            Err(e) => {
                error!(error = %e, "failed to accept ssh connection");
            }
        }
    }
}
