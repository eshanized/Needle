// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::client::SupabaseClient;
use crate::models::User;
use needle_common::error::{NeedleError, Result};
use serde_json::json;

pub async fn find_by_email(client: &SupabaseClient, email: &str) -> Result<Option<User>> {
    let response = client
        .select(
            "users",
            &[("email", &format!("eq.{email}")), ("limit", "1")],
        )
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let users: Vec<User> =
        serde_json::from_value(response).map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(users.into_iter().next())
}

pub async fn find_by_id(client: &SupabaseClient, id: &str) -> Result<Option<User>> {
    let response = client
        .select("users", &[("id", &format!("eq.{id}")), ("limit", "1")])
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let users: Vec<User> =
        serde_json::from_value(response).map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(users.into_iter().next())
}

pub async fn create(
    client: &SupabaseClient,
    email: &str,
    username: &str,
    password_hash: &str,
    auth_provider: &str,
) -> Result<User> {
    let body = json!({
        "email": email,
        "username": username,
        "password_hash": password_hash,
        "auth_provider": auth_provider,
        "tier": "free",
    });

    let response = client
        .insert("users", &body)
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let users: Vec<User> =
        serde_json::from_value(response).map_err(|e| NeedleError::Supabase(e.to_string()))?;

    users
        .into_iter()
        .next()
        .ok_or_else(|| NeedleError::Supabase("insert returned no rows".to_string()))
}
