// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::client::SupabaseClient;
use crate::models::TunnelRequest;
use needle_common::error::{NeedleError, Result};
use serde_json::json;

/// Fetches recent requests for a tunnel, ordered newest first.
/// Used by the traffic inspector view.
pub async fn find_recent(
    client: &SupabaseClient,
    tunnel_id: &str,
    limit: usize,
) -> Result<Vec<TunnelRequest>> {
    let value = client
        .select(
            "tunnel_requests",
            &[
                ("tunnel_id", &format!("eq.{tunnel_id}")),
                ("order", "timestamp.desc"),
                ("limit", &limit.to_string()),
            ],
        )
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let requests: Vec<TunnelRequest> =
        serde_json::from_value(value).map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(requests)
}

/// Logs a request that passed through a tunnel. Called by the
/// proxy layer after forwarding is complete.
pub async fn log_request(
    client: &SupabaseClient,
    tunnel_id: &str,
    method: &str,
    path: &str,
    status_code: u16,
    latency_ms: u32,
    request_size: usize,
    response_size: usize,
    client_ip: Option<&str>,
) -> Result<()> {
    let body = json!({
        "tunnel_id": tunnel_id,
        "method": method,
        "path": path,
        "status_code": status_code,
        "latency_ms": latency_ms,
        "request_size": request_size,
        "response_size": response_size,
        "client_ip": client_ip,
    });

    client
        .insert("tunnel_requests", &body)
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(())
}
