// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::error;

use crate::middleware::auth::Claims;
use crate::state::AppState;
use needle_db::queries::api_keys;

#[derive(Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateKeyResponse {
    pub key: String,
    pub prefix: String,
    pub name: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct KeyInfo {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub created_at: String,
    pub last_used: Option<String>,
}

/// Lists all API keys for the authenticated user.
/// We only return the prefix (first 8 chars), never the full key.
pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let user_id = claims.sub.to_string();
    match api_keys::find_by_user(&state.db, &user_id).await {
        Ok(keys) => {
            let infos: Vec<KeyInfo> = keys
                .iter()
                .map(|k| KeyInfo {
                    id: k.id.to_string(),
                    name: k.name.clone(),
                    prefix: k.key_prefix.clone(),
                    created_at: k.created_at.to_rfc3339(),
                    last_used: k.last_used.map(|t| t.to_rfc3339()),
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({ "keys": infos }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "failed to list api keys");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to list keys" })),
            )
                .into_response()
        }
    }
}

/// Creates a new API key. Generates a random 32-byte key,
/// stores a SHA-256 hash of it, and returns the plaintext key
/// exactly once. The user must save it -- we can never show it again.
pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateKeyRequest>,
) -> impl IntoResponse {
    let raw_key = generate_key();
    let prefix = raw_key[..8].to_string();
    let hash = hash_key(&raw_key);

    let user_id = claims.sub.to_string();
    match api_keys::create(&state.db, &user_id, &payload.name, &hash, &prefix).await {
        Ok(key) => {
            let resp = CreateKeyResponse {
                key: raw_key,
                prefix,
                name: key.name,
                id: key.id.to_string(),
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => {
            error!(error = %e, "failed to create api key");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to create key" })),
            )
                .into_response()
        }
    }
}

/// Deletes an API key by ID. Only the owner can delete their keys.
pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let user_id = claims.sub.to_string();
    match api_keys::delete(&state.db, &user_id, &key_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            error!(error = %e, "failed to delete api key");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to delete key" })),
            )
                .into_response()
        }
    }
}

/// Generates a random API key as a hex string (64 hex chars = 32 bytes).
fn generate_key() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    hex::encode(bytes)
}

/// Hashes an API key with SHA-256 for storage.
fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}
