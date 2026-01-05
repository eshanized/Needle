// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use needle_common::error::{NeedleError, Result};
use needle_db::client::SupabaseClient;
use needle_db::models::TunnelRequest;
use serde_json::json;

/// Fetches recent requests for a tunnel, ordered newest first.
/// Used by the traffic inspector view.
pub async fn find_recent(
    client: &SupabaseClient,
    tunnel_id: &str,
    limit: usize,
) -> Result<Vec<TunnelRequest>> {
    let requests: Vec<TunnelRequest> = client
        .from("tunnel_requests")
        .select("*")
        .eq("tunnel_id", tunnel_id)
        .order("timestamp", false)
        .limit(limit)
        .execute()
        .await?
        .json()
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(requests)
}

/// Logs a request that passed through a tunnel. Called by the
/// proxy layer after forwarding is complete so we have the
/// response status and latency.
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
        .from("tunnel_requests")
        .insert(&body)
        .execute()
        .await?;

    Ok(())
}
