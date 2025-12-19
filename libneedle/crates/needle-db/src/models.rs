// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub auth_provider: String,
    pub tier: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tunnel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subdomain: String,
    pub custom_domain: Option<String>,
    pub target_port: i32,
    pub protocol: String,
    pub is_active: bool,
    pub is_persistent: bool,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelRequest {
    pub id: Uuid,
    pub tunnel_id: Uuid,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub latency_ms: i32,
    pub request_headers: Option<serde_json::Value>,
    pub response_headers: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
