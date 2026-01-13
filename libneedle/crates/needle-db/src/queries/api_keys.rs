// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use needle_common::error::{NeedleError, Result};
use crate::client::SupabaseClient;
use crate::models::ApiKey;
use serde_json::json;
use tracing::info;

/// Finds all API keys belonging to a user. We only return the prefix
/// and metadata, never the full key (which we don't store).
pub async fn find_by_user(client: &SupabaseClient, user_id: &str) -> Result<Vec<ApiKey>> {
    let value = client
        .select("api_keys", &[("user_id", &format!("eq.{user_id}"))])
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let keys: Vec<ApiKey> = serde_json::from_value(value)
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(keys)
}

/// Creates a new API key record. The caller is responsible for
/// hashing the key before passing it in -- we never store plaintext keys.
pub async fn create(
    client: &SupabaseClient,
    user_id: &str,
    name: &str,
    key_hash: &str,
    key_prefix: &str,
) -> Result<ApiKey> {
    let body = json!({
        "user_id": user_id,
        "name": name,
        "key_hash": key_hash,
        "key_prefix": key_prefix,
    });

    let value = client
        .insert("api_keys", &body)
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    // supabase returns an array, grab the first item
    let keys: Vec<ApiKey> = serde_json::from_value(value)
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    keys.into_iter()
        .next()
        .ok_or_else(|| NeedleError::Supabase("insert returned no data".to_string()))
}

/// Deletes an API key, but only if it belongs to the given user.
pub async fn delete(
    client: &SupabaseClient,
    user_id: &str,
    key_id: &str,
) -> Result<()> {
    client
        .delete(
            "api_keys",
            &[
                ("id", &format!("eq.{key_id}")),
                ("user_id", &format!("eq.{user_id}")),
            ],
        )
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    info!(user_id = %user_id, key_id = %key_id, "api key deleted");
    Ok(())
}

/// Looks up an API key by its hash for authentication purposes.
pub async fn find_by_hash(
    client: &SupabaseClient,
    key_hash: &str,
) -> Result<Option<ApiKey>> {
    let value = client
        .select("api_keys", &[("key_hash", &format!("eq.{key_hash}"))])
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let keys: Vec<ApiKey> = serde_json::from_value(value)
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(keys.into_iter().next())
}
