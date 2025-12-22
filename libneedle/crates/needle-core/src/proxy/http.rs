// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::debug;

const PROXY_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const PROXY_RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESPONSE_SIZE: usize = 50 * 1024 * 1024; // 50MB

/// Forwards an incoming HTTP request to a tunnel's internal listener.
///
/// Here's what happens step by step:
/// 1. We open a TCP connection to the tunnel's local listener address
///    (127.0.0.1:random_port)
/// 2. We serialize the HTTP request and write it over that TCP connection
/// 3. The SSH layer picks it up and sends it through the SSH channel
///    to the client's local app
/// 4. The response comes back the same way, we read it and return it
///
/// This is basically a minimal HTTP/1.1 reverse proxy that works at the
/// byte level rather than using a full HTTP client library. That gives
/// us more control over streaming and keeps the overhead low.
pub async fn forward_request(
    bind_addr: SocketAddr,
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, ProxyError> {
    let stream = timeout(PROXY_CONNECT_TIMEOUT, TcpStream::connect(bind_addr))
        .await
        .map_err(|_| ProxyError::ConnectTimeout)?
        .map_err(ProxyError::Connect)?;

    let (mut read_half, mut write_half) = stream.into_split();

    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    let body_bytes = req
        .into_body()
        .collect()
        .await
        .map_err(|e| ProxyError::ReadBody(e.to_string()))?
        .to_bytes();

    let mut raw_request = format!(
        "{} {} HTTP/1.1\r\n",
        method,
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );

    for (name, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            raw_request.push_str(&format!("{}: {}\r\n", name, v));
        }
    }

    if !body_bytes.is_empty() {
        raw_request.push_str(&format!("content-length: {}\r\n", body_bytes.len()));
    }

    raw_request.push_str("\r\n");

    write_half
        .write_all(raw_request.as_bytes())
        .await
        .map_err(ProxyError::Write)?;

    if !body_bytes.is_empty() {
        write_half
            .write_all(&body_bytes)
            .await
            .map_err(ProxyError::Write)?;
    }

    write_half.flush().await.map_err(ProxyError::Write)?;

    let mut response_buf = Vec::with_capacity(8192);
    let mut temp = [0u8; 8192];

    let read_result = timeout(PROXY_RESPONSE_TIMEOUT, async {
        loop {
            let n = read_half.read(&mut temp).await.map_err(ProxyError::Read)?;
            if n == 0 {
                break;
            }
            response_buf.extend_from_slice(&temp[..n]);
            if response_buf.len() > MAX_RESPONSE_SIZE {
                return Err(ProxyError::ResponseTooLarge);
            }

            if response_buf.windows(4).any(|w| w == b"\r\n\r\n") && response_buf.len() > 100 {
                break;
            }
        }
        Ok(())
    })
    .await;

    match read_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err(ProxyError::ResponseTimeout),
    }

    debug!(
        bytes = response_buf.len(),
        "received proxy response"
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from(response_buf)))
        .expect("valid response"))
}

/// Builds a plain-text error response for when the proxy can't reach
/// the tunnel backend. We keep it simple so the client gets useful
/// feedback without exposing internal details.
pub fn error_response(status: StatusCode, message: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(Full::new(Bytes::from(message.to_string())))
        .expect("valid error response")
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("timed out connecting to tunnel backend")]
    ConnectTimeout,

    #[error("failed to connect to tunnel backend: {0}")]
    Connect(std::io::Error),

    #[error("failed to read request body: {0}")]
    ReadBody(String),

    #[error("failed to write to tunnel backend: {0}")]
    Write(std::io::Error),

    #[error("failed to read from tunnel backend: {0}")]
    Read(std::io::Error),

    #[error("tunnel response timed out")]
    ResponseTimeout,

    #[error("response exceeded size limit")]
    ResponseTooLarge,
}
