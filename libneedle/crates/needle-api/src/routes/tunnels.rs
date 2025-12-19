// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::middleware::auth::Claims;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateTunnelRequest {
    pub subdomain: Option<String>,
    pub target_port: u32,
    pub protocol: Option<String>,
    pub is_persistent: Option<bool>,
}

pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let tunnels =
        needle_db::queries::tunnels::find_by_user(&state.db, &claims.sub.to_string()).await;

    match tunnels {
        Ok(list) => (StatusCode::OK, Json(json!({ "tunnels": list }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(_payload): Json<CreateTunnelRequest>,
) -> impl IntoResponse {
    let mut manager = state.tunnel_manager.write().await;

    let tunnel = manager.create("api", claims.sub).await;

    match tunnel {
        Ok(t) => {
            info!(subdomain = %t.subdomain, "tunnel created via api");
            (
                StatusCode::CREATED,
                Json(json!({
                    "subdomain": t.subdomain,
                    "url": format!("https://{}.{}", t.subdomain, state.domain),
                    "bind_addr": t.bind_addr.to_string(),
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(subdomain): Path<String>,
) -> impl IntoResponse {
    let mut manager = state.tunnel_manager.write().await;

    match manager.remove(&subdomain).await {
        Ok(()) => {
            info!(subdomain = %subdomain, "tunnel deleted via api");
            StatusCode::NO_CONTENT
        }
        Err(_) => StatusCode::NOT_FOUND,
    }
}
