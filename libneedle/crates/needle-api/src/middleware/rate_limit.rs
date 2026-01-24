// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::Json;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

use needle_common::rate_limit::RateLimiter;

/// Shared rate limiter state keyed by client IP address.
/// Each IP gets its own bucket so one abusive client can't
/// starve others.
pub type RateLimiterMap = Arc<RwLock<std::collections::HashMap<String, RateLimiter>>>;

const REQUESTS_PER_SECOND: f64 = 10.0;
const BURST_SIZE: f64 = 30.0;

/// Creates a new empty rate limiter map.
pub fn new_rate_limiter_map() -> RateLimiterMap {
    Arc::new(RwLock::new(std::collections::HashMap::new()))
}

/// Axum middleware that enforces per-IP rate limiting using
/// the token bucket algorithm from needle-common. If a client
/// exceeds the limit, they get a 429 with a helpful message.
pub async fn rate_limit(
    State(limiter_map): State<RateLimiterMap>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = addr.ip().to_string();

    let allowed = {
        let mut map = limiter_map.write().await;
        let limiter = map
            .entry(ip.clone())
            .or_insert_with(|| RateLimiter::new(REQUESTS_PER_SECOND, BURST_SIZE));
        limiter.allow()
    };

    if !allowed {
        warn!(ip = %ip, "rate limit exceeded");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "too many requests, please slow down"
            })),
        )
            .into_response();
    }

    next.run(request).await
}
