// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use tracing::error;

use crate::middleware::auth::Claims;
use crate::state::AppState;
use needle_db::queries::requests;

#[derive(Deserialize)]
pub struct InspectorQuery {
    pub limit: Option<usize>,
}

/// Fetches recent requests for a specific tunnel.
/// Useful for debugging what's flowing through the tunnel in real time.
/// Defaults to 50 requests if no limit is specified.
pub async fn list_requests(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(tunnel_id): Path<String>,
    Query(query): Query<InspectorQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(200);

    match requests::find_recent(&state.db, &tunnel_id, limit).await {
        Ok(reqs) => {
            (StatusCode::OK, Json(serde_json::json!({ "requests": reqs }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "failed to fetch tunnel requests");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "failed to fetch requests" }))).into_response()
        }
    }
}
