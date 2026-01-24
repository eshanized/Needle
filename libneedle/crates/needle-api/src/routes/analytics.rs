// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use tracing::error;

use crate::middleware::auth::Claims;
use crate::state::AppState;
use needle_db::queries::analytics;

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub days: Option<usize>,
}

/// Fetches daily analytics for a specific tunnel.
/// Defaults to 30 days if no period is specified.
pub async fn tunnel_stats(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(tunnel_id): Path<String>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let days = query.days.unwrap_or(30).min(90);

    match analytics::get_daily_stats(&state.db, &tunnel_id, days).await {
        Ok(stats) => (StatusCode::OK, Json(serde_json::json!({ "stats": stats }))).into_response(),
        Err(e) => {
            error!(error = %e, "failed to fetch tunnel analytics");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to fetch analytics" })),
            )
                .into_response()
        }
    }
}

/// Fetches a summary of analytics across all tunnels for the user.
/// Powers the dashboard overview cards.
pub async fn user_summary(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = claims.sub.to_string();
    match analytics::get_user_summary(&state.db, &user_id).await {
        Ok(summary) => (
            StatusCode::OK,
            Json(serde_json::json!({ "summary": summary })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "failed to fetch user analytics");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to fetch analytics" })),
            )
                .into_response()
        }
    }
}
