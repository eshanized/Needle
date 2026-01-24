// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::middleware::auth::Claims;
use crate::state::AppState;

const TOKEN_EXPIRY_HOURS: i64 = 24;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let existing = needle_db::queries::users::find_by_email(&state.db, &payload.email).await;
    if let Ok(Some(_)) = existing {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "email already registered" })),
        );
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "failed to hash password" })),
            );
        }
    };

    let user = needle_db::queries::users::create(
        &state.db,
        &payload.email,
        &payload.username,
        &password_hash,
        "email",
    )
    .await;

    match user {
        Ok(user) => {
            info!(email = %user.email, "new user registered");

            let token = create_token(&state.jwt_secret, user.id, &user.email, &user.tier);
            match token {
                Ok(t) => (
                    StatusCode::CREATED,
                    Json(json!({
                        "token": t,
                        "user": {
                            "id": user.id,
                            "email": user.email,
                            "username": user.username,
                            "tier": user.tier,
                        }
                    })),
                ),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "failed to create token" })),
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match needle_db::queries::users::find_by_email(&state.db, &payload.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "invalid credentials" })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            );
        }
    };

    // Verify password hash
    if !verify_password(&payload.password, &user.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid credentials" })),
        );
    }

    info!(email = %user.email, "user logged in");

    let token = match create_token(&state.jwt_secret, user.id, &user.email, &user.tier) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "failed to create token" })),
            );
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "token": token,
            "user": {
                "id": user.id,
                "email": user.email,
                "username": user.username,
                "tier": user.tier,
            }
        })),
    )
}

/// Builds a signed JWT with user identity and tier info packed into the
/// claims. The token expires after TOKEN_EXPIRY_HOURS so clients need
/// to refresh periodically. We use HS256 here since the secret stays
/// server-side -- if we ever need third-party verification we'd switch
/// to RS256 with a public key.
fn create_token(
    secret: &str,
    user_id: uuid::Uuid,
    email: &str,
    tier: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiry = Utc::now()
        .checked_add_signed(chrono::Duration::hours(TOKEN_EXPIRY_HOURS))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        tier: tier.to_string(),
        exp: expiry,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::password_hash::SaltString;
    use argon2::password_hash::rand_core::OsRng;
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::password_hash::PasswordHash;
    use argon2::{Argon2, PasswordVerifier};

    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
