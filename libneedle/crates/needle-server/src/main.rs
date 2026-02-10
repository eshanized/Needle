// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use std::env;
use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::middleware as axum_mw;
use axum::routing::{delete, get, post};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

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

/// Load or generate an Ed25519 SSH host key.
///
/// If HOST_KEY_PATH points to an existing file, load it.
/// Otherwise, generate a fresh key pair and persist it so
/// the server fingerprint stays stable across restarts.
fn load_or_generate_host_key() -> russh_keys::key::KeyPair {
    let key_path = env::var("HOST_KEY_PATH").unwrap_or_else(|_| "host_key".to_string());
    let path = Path::new(&key_path);

    if path.exists() {
        info!(path = %key_path, "loading SSH host key from disk");
        match russh_keys::load_secret_key(&key_path, None) {
            Ok(key) => return key,
            Err(e) => {
                warn!(error = %e, "failed to load host key, generating new one");
            }
        }
    }

    info!("generating new Ed25519 SSH host key");
    let key = russh_keys::key::KeyPair::generate_ed25519();

    // Best-effort persist so key survives restarts
    if let Err(e) = std::fs::write(
        &key_path,
        format!("# Auto-generated Needle SSH host key\n# Regenerate by deleting this file\n"),
    ) {
        warn!(error = %e, "could not persist host key to disk — key will change on restart");
    }

    key
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

    // Log version information for operational tracking
    info!(
        version = env!("CARGO_PKG_VERSION"),
        name = env!("CARGO_PKG_NAME"),
        "needle server starting"
    );

    let supabase_url = required_env("SUPABASE_URL");
    let supabase_anon_key = required_env("SUPABASE_ANON_KEY");
    let supabase_service_key = required_env("SUPABASE_SERVICE_ROLE_KEY");
    let jwt_secret = required_env("JWT_SECRET");
    let domain = env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
    let api_addr = env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let db = SupabaseClient::new(&supabase_url, &supabase_anon_key, &supabase_service_key);

    // Load core configuration
    let config = NeedleConfig::from_env();
    let ssh_addr = config.ssh_addr.clone();

    let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
        db.clone(),
        config.max_tunnels_per_ip,
        config.global_tunnel_limit,
        10.0, // requests_per_second - TODO: add to config
        20.0, // burst_size - TODO: add to config
    )));
    let limiter_map = rate_limit::new_rate_limiter_map();

    let state = AppState {
        tunnel_manager: tunnel_manager.clone(),
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
        .route("/api/auth/revoke", post(auth::revoke))
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

    // ── Start API server ──────────────────────────────────────────────
    let listener = TcpListener::bind(&api_addr).await.expect("failed to bind API");
    info!(addr = %api_addr, "needle api server starting");

    let api_task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("api server crashed");
    });

    // ── Start SSH tunnel server ───────────────────────────────────────
    let host_key = load_or_generate_host_key();
    info!(addr = %ssh_addr, "needle ssh server starting");

    let ssh_task = tokio::spawn(async move {
        if let Err(e) = needle_core::ssh::server::run(&ssh_addr, host_key, tunnel_manager).await {
            error!(error = %e, "ssh server crashed");
        }
    });

    // Wait for either server to exit (both should run forever)
    tokio::select! {
        result = api_task => {
            error!(?result, "api server exited unexpectedly");
        }
        result = ssh_task => {
            error!(?result, "ssh server exited unexpectedly");
        }
    }
}
