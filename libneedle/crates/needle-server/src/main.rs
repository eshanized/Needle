// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use std::env;
use std::sync::Arc;

use axum::middleware as axum_mw;
use axum::routing::{delete, get, post};
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use needle_api::middleware::auth::require_auth;
use needle_api::routes::{auth, health, tunnels};
use needle_api::state::AppState;
use needle_core::tunnel::manager::TunnelManager;
use needle_db::client::SupabaseClient;

fn required_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "needle=debug,tower_http=debug".into()),
        )
        .init();

    let supabase_url = required_env("SUPABASE_URL");
    let supabase_anon_key = required_env("SUPABASE_ANON_KEY");
    let supabase_service_key = required_env("SUPABASE_SERVICE_ROLE_KEY");
    let jwt_secret = required_env("JWT_SECRET");
    let domain = env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
    let api_addr = env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let db = SupabaseClient::new(&supabase_url, &supabase_anon_key, &supabase_service_key);
    let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(db.clone())));

    let state = AppState {
        tunnel_manager,
        db,
        jwt_secret,
        domain,
    };

    let public_routes = Router::new()
        .route("/health", get(health::check))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login));

    let protected_routes = Router::new()
        .route("/api/tunnels", get(tunnels::list).post(tunnels::create))
        .route("/api/tunnels/{subdomain}", delete(tunnels::delete))
        .layer(axum_mw::from_fn_with_state(state.clone(), require_auth));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind(&api_addr).await.expect("failed to bind");
    info!(addr = %api_addr, "needle api server starting");

    axum::serve(listener, app)
        .await
        .expect("server crashed");
}
