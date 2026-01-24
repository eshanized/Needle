// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::client::SupabaseClient;
use needle_common::error::{NeedleError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyAnalytics {
    pub id: String,
    pub tunnel_id: String,
    pub date: String,
    pub total_requests: i64,
    pub total_bytes_in: i64,
    pub total_bytes_out: i64,
    pub avg_latency_ms: i32,
    pub error_count: i32,
    pub unique_ips: i32,
}

/// Fetches daily analytics for a tunnel over a date range.
/// Used by the analytics charts on the frontend.
pub async fn get_daily_stats(
    client: &SupabaseClient,
    tunnel_id: &str,
    days: usize,
) -> Result<Vec<DailyAnalytics>> {
    let value = client
        .select(
            "analytics_daily",
            &[
                ("tunnel_id", &format!("eq.{tunnel_id}")),
                ("order", "date.desc"),
                ("limit", &days.to_string()),
            ],
        )
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let stats: Vec<DailyAnalytics> =
        serde_json::from_value(value).map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(stats)
}

/// Fetches summary analytics across all tunnels for a user.
/// Powers the dashboard overview cards.
pub async fn get_user_summary(
    client: &SupabaseClient,
    user_id: &str,
) -> Result<UserAnalyticsSummary> {
    // fetch the user's tunnels first
    let tunnels_value = client
        .select(
            "tunnels",
            &[("user_id", &format!("eq.{user_id}")), ("select", "id")],
        )
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let tunnel_ids: Vec<String> = tunnels_value
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|t| t.get("id").and_then(|v| v.as_str()).map(String::from))
        .collect();

    if tunnel_ids.is_empty() {
        return Ok(UserAnalyticsSummary::default());
    }

    // get the last 7 days of analytics for all user tunnels
    let mut total_requests: i64 = 0;
    let mut total_bytes: i64 = 0;

    for tid in &tunnel_ids {
        let stats = get_daily_stats(client, tid, 7).await?;
        for day in &stats {
            total_requests += day.total_requests;
            total_bytes += day.total_bytes_in + day.total_bytes_out;
        }
    }

    Ok(UserAnalyticsSummary {
        total_tunnels: tunnel_ids.len() as i64,
        requests_7d: total_requests,
        bytes_7d: total_bytes,
    })
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserAnalyticsSummary {
    pub total_tunnels: i64,
    pub requests_7d: i64,
    pub bytes_7d: i64,
}
