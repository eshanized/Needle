// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NeedleError {
    #[error("tunnel not found: {0}")]
    TunnelNotFound(String),

    #[error("subdomain already taken: {0}")]
    SubdomainTaken(String),

    #[error("subdomain is invalid: {0}")]
    InvalidSubdomain(String),

    #[error("rate limit exceeded for tunnel: {0}")]
    RateLimited(String),

    #[error("max tunnels reached for this ip")]
    MaxTunnelsPerIp,

    #[error("server is at capacity")]
    ServerAtCapacity,

    #[error("ip is temporarily blocked until {0}")]
    IpBlocked(String),

    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("insufficient permissions: {0}")]
    Forbidden(String),

    #[error("ssh handshake timed out")]
    SshHandshakeTimeout,

    #[error("tunnel has expired")]
    TunnelExpired,

    #[error("request body too large")]
    BodyTooLarge,

    #[error("websocket transfer limit exceeded")]
    WebSocketTransferLimit,

    #[error("supabase error: {0}")]
    Supabase(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, NeedleError>;
