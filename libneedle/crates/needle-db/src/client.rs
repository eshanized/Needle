// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use reqwest::Client;
use serde_json::Value;
use tracing::debug;

/// Wraps the Supabase REST API so the rest of the codebase doesn't need
/// to know about HTTP details. All database reads and writes go through
/// this client, which handles auth headers and base URL construction.
#[derive(Clone)]
pub struct SupabaseClient {
    http: Client,
    base_url: String,
    api_key: String,
    service_role_key: String,
}

impl SupabaseClient {
    pub fn new(url: &str, api_key: &str, service_role_key: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: format!("{url}/rest/v1"),
            api_key: api_key.to_string(),
            service_role_key: service_role_key.to_string(),
        }
    }

    /// Runs a SELECT query against a Supabase table. The `query_params`
    /// get turned into PostgREST query string parameters, so you can do
    /// filtering like `("subdomain", "eq.brave-eagle-a1b2c3d4")`.
    pub async fn select(
        &self,
        table: &str,
        query_params: &[(&str, &str)],
    ) -> Result<Value, reqwest::Error> {
        debug!(table, "selecting from supabase");

        let response = self
            .http
            .get(format!("{}/{table}", self.base_url))
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .query(query_params)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response)
    }

    /// Inserts one or more rows into a table. The `body` should be a JSON
    /// object (single row) or array (multiple rows). Returns the inserted
    /// data with any database-generated fields like `id` and timestamps.
    pub async fn insert(
        &self,
        table: &str,
        body: &Value,
    ) -> Result<Value, reqwest::Error> {
        debug!(table, "inserting into supabase");

        let response = self
            .http
            .post(format!("{}/{table}", self.base_url))
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response)
    }

    /// Updates rows matching the given filters. The `query_params` define
    /// which rows to update (PostgREST filter syntax), and `body` contains
    /// the fields to change.
    pub async fn update(
        &self,
        table: &str,
        query_params: &[(&str, &str)],
        body: &Value,
    ) -> Result<Value, reqwest::Error> {
        debug!(table, "updating in supabase");

        let response = self
            .http
            .patch(format!("{}/{table}", self.base_url))
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .query(query_params)
            .json(body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response)
    }

    /// Deletes rows matching the given filters. Be careful with this --
    /// without filters it would delete everything in the table.
    pub async fn delete(
        &self,
        table: &str,
        query_params: &[(&str, &str)],
    ) -> Result<Value, reqwest::Error> {
        debug!(table, "deleting from supabase");

        let response = self
            .http
            .delete(format!("{}/{table}", self.base_url))
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .query(query_params)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response)
    }
}
