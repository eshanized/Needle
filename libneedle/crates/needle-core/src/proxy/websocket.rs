// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tracing::{debug, info};

const WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const WS_IDLE_TIMEOUT: Duration = Duration::from_secs(300);
const WS_MAX_TRANSFER: usize = 100 * 1024 * 1024; // 100MB per session

/// Bridges a WebSocket connection between the external client and the
/// tunnel's local listener.
///
/// Unlike normal HTTP proxying, WebSocket needs bidirectional byte
/// copying that runs until one side closes. We track total bytes
/// transferred and cut things off if the session exceeds our limit.
///
/// The idle timeout catches abandoned connections -- if neither side
/// sends anything for 5 minutes, we assume the session is dead and
/// clean up.
pub async fn bridge(
    bind_addr: SocketAddr,
    client_stream: TcpStream,
) -> Result<WebSocketStats, WebSocketError> {
    let tunnel_stream = timeout(WS_CONNECT_TIMEOUT, TcpStream::connect(bind_addr))
        .await
        .map_err(|_| WebSocketError::ConnectTimeout)?
        .map_err(WebSocketError::Connect)?;

    let (mut tunnel_read, mut tunnel_write) = tunnel_stream.into_split();
    let (mut client_read, mut client_write) = client_stream.into_split();

    let upstream = tokio::spawn(async move {
        let mut buf = [0u8; 8192];
        let mut bytes = 0usize;

        loop {
            let read_result = timeout(WS_IDLE_TIMEOUT, client_read.read(&mut buf)).await;

            match read_result {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    bytes += n;
                    if bytes > WS_MAX_TRANSFER {
                        debug!("websocket upstream transfer limit reached");
                        break;
                    }
                    if tunnel_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => {
                    debug!("websocket upstream idle timeout");
                    break;
                }
            }
        }

        bytes
    });

    let downstream = tokio::spawn(async move {
        let mut buf = [0u8; 8192];
        let mut bytes = 0usize;

        loop {
            let read_result = timeout(WS_IDLE_TIMEOUT, tunnel_read.read(&mut buf)).await;

            match read_result {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    bytes += n;
                    if bytes > WS_MAX_TRANSFER {
                        debug!("websocket downstream transfer limit reached");
                        break;
                    }
                    if client_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => {
                    debug!("websocket downstream idle timeout");
                    break;
                }
            }
        }

        bytes
    });

    let total_up = upstream.await.unwrap_or(0);
    let total_down = downstream.await.unwrap_or(0);

    info!(
        up_bytes = total_up,
        down_bytes = total_down,
        "websocket session ended"
    );

    Ok(WebSocketStats {
        bytes_up: total_up,
        bytes_down: total_down,
    })
}

#[derive(Debug)]
pub struct WebSocketStats {
    pub bytes_up: usize,
    pub bytes_down: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("timed out connecting to tunnel for websocket")]
    ConnectTimeout,

    #[error("failed to connect to tunnel for websocket: {0}")]
    Connect(std::io::Error),
}
