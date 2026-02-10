// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{DecodingKey, Validation, decode};
use needle_core::metrics;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user ID
    pub email: String,
    pub tier: String,
    pub exp: usize,  // expiration timestamp
    pub jti: String, // JWT ID for revocation
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
        .ok_or_else(|| {
            metrics::auth_failure("api", "missing_header");
            StatusCode::UNAUTHORIZED
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        metrics::auth_failure("api", "invalid_header_format");
        StatusCode::UNAUTHORIZED
    })?;

    let key = DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &key, &validation).map_err(|_| {
        metrics::auth_failure("api", "invalid_token");
        StatusCode::UNAUTHORIZED
    })?;

    let claims = token_data.claims;

    // Check if token is revoked
    match is_token_revoked(&state, &claims.jti).await {
        Ok(true) => {
            metrics::auth_failure("api", "token_revoked");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Ok(false) => {} // Token is valid, continue
        Err(e) => {
            tracing::error!(error = %e, "failed to check token revocation");
            metrics::error_occurred("revocation_check_failed");
            // Fail open - allow request if we can't check revocation
            // In production you might want to fail closed
        }
    }

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Check if a token has been revoked by querying the revoked_tokens table
async fn is_token_revoked(state: &AppState, jti: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let response = state
        .db
        .select("revoked_tokens", &[("jti", &format!("eq.{}", jti))])
        .await?;

    // If we got any results, the token is revoked
    Ok(response
        .as_array()
        .map(|arr| !arr.is_empty())
        .unwrap_or(false))
}
