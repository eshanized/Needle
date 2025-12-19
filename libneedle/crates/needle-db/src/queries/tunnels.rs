// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::client::SupabaseClient;
use crate::models::Tunnel;
use needle_common::error::{NeedleError, Result};
use serde_json::json;

pub async fn find_by_subdomain(
    client: &SupabaseClient,
    subdomain: &str,
) -> Result<Option<Tunnel>> {
    let response = client
        .select(
            "tunnels",
            &[("subdomain", &format!("eq.{subdomain}")), ("limit", "1")],
        )
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let tunnels: Vec<Tunnel> = serde_json::from_value(response)
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(tunnels.into_iter().next())
}

pub async fn find_by_user(client: &SupabaseClient, user_id: &str) -> Result<Vec<Tunnel>> {
    let response = client
        .select("tunnels", &[("user_id", &format!("eq.{user_id}"))])
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    serde_json::from_value(response).map_err(|e| NeedleError::Supabase(e.to_string()))
}

pub async fn create(
    client: &SupabaseClient,
    user_id: &str,
    subdomain: &str,
    target_port: i32,
    protocol: &str,
    is_persistent: bool,
) -> Result<Tunnel> {
    let body = json!({
        "user_id": user_id,
        "subdomain": subdomain,
        "target_port": target_port,
        "protocol": protocol,
        "is_active": true,
        "is_persistent": is_persistent,
    });

    let response = client
        .insert("tunnels", &body)
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    let tunnels: Vec<Tunnel> = serde_json::from_value(response)
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    tunnels
        .into_iter()
        .next()
        .ok_or_else(|| NeedleError::Supabase("insert returned no rows".to_string()))
}

pub async fn set_active(
    client: &SupabaseClient,
    subdomain: &str,
    active: bool,
) -> Result<()> {
    client
        .update(
            "tunnels",
            &[("subdomain", &format!("eq.{subdomain}"))],
            &json!({ "is_active": active }),
        )
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(())
}

pub async fn delete_by_id(client: &SupabaseClient, id: &str) -> Result<()> {
    client
        .delete("tunnels", &[("id", &format!("eq.{id}"))])
        .await
        .map_err(|e| NeedleError::Supabase(e.to_string()))?;

    Ok(())
}
