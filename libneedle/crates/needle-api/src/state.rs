// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use needle_core::tunnel::manager::TunnelManager;
use needle_db::client::SupabaseClient;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state that gets passed into every Axum handler.
/// We wrap mutables in Arc<RwLock<>> so handlers can read tunnel state
/// concurrently while still allowing writes when tunnels are created
/// or removed.
#[derive(Clone)]
pub struct AppState {
    pub tunnel_manager: Arc<RwLock<TunnelManager>>,
    pub db: SupabaseClient,
    pub jwt_secret: String,
    pub domain: String,
}
