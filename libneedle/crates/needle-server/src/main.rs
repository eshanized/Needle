// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use std::env;
use std::sync::Arc;

use axum::Router;
use axum::middleware as axum_mw;
use axum::routing::{delete, get, post};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use needle_api::middleware::auth::require_auth;
use needle_api::middleware::rate_limit;
use needle_api::routes::{analytics, api_keys, auth, health, inspector, metrics, tunnels};
use needle_api::state::AppState;
use needle_core::config::NeedleConfig;
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
    
    // Load core configuration
    let config = NeedleConfig::from_env();
    
    let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
        db.clone(),
        config.max_tunnels_per_ip,
        config.global_tunnel_limit,
        10.0,  // requests_per_second - TODO: add to config
        20.0,  // burst_size - TODO: add to config
    )));
    let limiter_map = rate_limit::new_rate_limiter_map();

    let state = AppState {
        tunnel_manager,
        db,
        jwt_secret,
        domain,
    };

    // public routes -- no auth needed
    let public_routes = Router::new()
        .route("/health", get(health::check))
        .route("/metrics", get(metrics::metrics))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login));

    // protected routes -- require valid JWT
    let protected_routes = Router::new()
        .route("/api/tunnels", get(tunnels::list).post(tunnels::create))
        .route("/api/tunnels/{subdomain}", delete(tunnels::delete))
        .route("/api/keys", get(api_keys::list).post(api_keys::create))
        .route("/api/keys/{key_id}", delete(api_keys::delete))
        .route(
            "/api/tunnels/{tunnel_id}/requests",
            get(inspector::list_requests),
        )
        .route(
            "/api/tunnels/{tunnel_id}/analytics",
            get(analytics::tunnel_stats),
        )
        .route("/api/analytics/summary", get(analytics::user_summary))
        .layer(axum_mw::from_fn_with_state(state.clone(), require_auth));

    // Configure CORS - default to localhost for dev, require explicit origin for prod
    let cors_origin =
        env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let cors = CorsLayer::new()
        .allow_origin(
            cors_origin
                .parse::<axum::http::HeaderValue>()
                .unwrap_or_else(|_| {
                    info!("invalid CORS_ORIGIN, using localhost");
                    "http://localhost:5173".parse().unwrap()
                }),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(axum_mw::from_fn_with_state(
            limiter_map,
            rate_limit::rate_limit,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind(&api_addr).await.expect("failed to bind");
    info!(addr = %api_addr, "needle api server starting");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("server crashed");
}
