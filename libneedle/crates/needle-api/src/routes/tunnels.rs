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
    Json(payload): Json<CreateTunnelRequest>,
) -> impl IntoResponse {
    // Validate and handle custom subdomain if provided
    let custom_subdomain = if let Some(ref subdomain) = payload.subdomain {
        // Validate custom subdomain format
        if !needle_common::subdomain::is_valid_custom(subdomain) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid subdomain format" })),
            )
                .into_response();
        }

        // Check if subdomain is already taken
        match needle_db::queries::tunnels::find_by_subdomain(&state.db, subdomain).await {
            Ok(Some(_)) => {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "subdomain already taken" })),
                )
                    .into_response();
            }
            Ok(None) => {}
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                )
                    .into_response();
            }
        }

        Some(subdomain.clone())
    } else {
        None
    };

    let mut manager = state.tunnel_manager.write().await;

    let tunnel = manager
        .create(
            &claims.sub.to_string(),
            claims.sub,
            custom_subdomain,
            payload.target_port.unwrap_or(80) as i32,
            &payload.protocol.unwrap_or_else(|| "http".to_string()),
            payload.is_persistent.unwrap_or(false),
        )
        .await;

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
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(subdomain): Path<String>,
) -> impl IntoResponse {
    // Verify tunnel exists and belongs to user
    match needle_db::queries::tunnels::find_by_subdomain(&state.db, &subdomain).await {
        Ok(Some(tunnel)) => {
            if tunnel.user_id != claims.sub {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({ "error": "you do not own this tunnel" })),
                )
                    .into_response();
            }
        }
        Ok(None) => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    }

    let mut manager = state.tunnel_manager.write().await;

    match manager.remove(&subdomain).await {
        Ok(()) => {
            info!(subdomain = %subdomain, "tunnel deleted via api");
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
