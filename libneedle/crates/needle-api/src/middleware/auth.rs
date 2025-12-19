// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub tier: String,
    pub exp: usize,
}

/// Pulls the JWT from the Authorization header, validates it, and
/// stashes the decoded claims into request extensions so downstream
/// handlers can access the authenticated user without re-parsing.
///
/// If the token is missing or invalid, we short-circuit with a 401.
/// This runs as Axum middleware on all protected routes.
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let key = DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}
