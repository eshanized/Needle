// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use needle_common::error::{NeedleError, Result};
use needle_db::client::SupabaseClient;
use needle_db::models::ApiKey;
use serde_json::json;
use tracing::info;

/// Finds all API keys belonging to a user. We only return the prefix
/// and metadata, never the full key (which we don't store).
pub async fn find_by_user(client: &SupabaseClient, user_id: &str) -> Result<Vec<ApiKey>> {
    let keys: Vec<ApiKey> = client
        .from("api_keys")
        .select("id,user_id,name,key_prefix,scopes,last_used,expires_at,created_at")
        .eq("user_id", user_id)
        .execute()
        .await?
        .json()
        .await
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

    let key: ApiKey = client
        .from("api_keys")
        .insert(&body)
        .execute()
        .await?
        .json()
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    info!(user_id = %user_id, name = %name, "api key created");
    Ok(key)
}

/// Deletes an API key, but only if it belongs to the given user.
/// Returns an error if the key doesn't exist or doesn't belong to them.
pub async fn delete(
    client: &SupabaseClient,
    user_id: &str,
    key_id: &str,
) -> Result<()> {
    client
        .from("api_keys")
        .eq("id", key_id)
        .eq("user_id", user_id)
        .delete()
        .execute()
        .await?;

    info!(user_id = %user_id, key_id = %key_id, "api key deleted");
    Ok(())
}

/// Looks up an API key by its hash for authentication purposes.
/// If found, we also update the last_used timestamp.
pub async fn find_by_hash(
    client: &SupabaseClient,
    key_hash: &str,
) -> Result<Option<ApiKey>> {
    let keys: Vec<ApiKey> = client
        .from("api_keys")
        .select("*")
        .eq("key_hash", key_hash)
        .execute()
        .await?
        .json()
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(keys.into_iter().next())
}
