// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::env;
use tracing::info;

const DEFAULT_API_ADDR: &str = "0.0.0.0:3000";
const DEFAULT_SSH_ADDR: &str = "0.0.0.0:2222";
const DEFAULT_DOMAIN: &str = "localhost";
const DEFAULT_MAX_TUNNELS_PER_IP: usize = 5;
const DEFAULT_GLOBAL_TUNNEL_LIMIT: usize = 1000;

/// Runtime configuration assembled from environment variables.
///
/// We don't use a config file on purpose -- env vars play nicely with
/// containers and twelve-factor deployments. Every value has a sensible
/// default so you can start the server with nothing but the Supabase
/// credentials.
#[derive(Debug, Clone, Deserialize)]
pub struct NeedleConfig {
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_service_key: String,
    pub jwt_secret: String,
    pub domain: String,
    pub api_addr: String,
    pub ssh_addr: String,
    pub max_tunnels_per_ip: usize,
    pub global_tunnel_limit: usize,
}

impl NeedleConfig {
    /// Reads config from the environment. Panics early with a clear
    /// message if a required variable is missing -- there's no point
    /// starting the server without Supabase credentials.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let config = Self {
            supabase_url: required("SUPABASE_URL"),
            supabase_anon_key: required("SUPABASE_ANON_KEY"),
            supabase_service_key: required("SUPABASE_SERVICE_ROLE_KEY"),
            jwt_secret: required("JWT_SECRET"),
            domain: env::var("DOMAIN").unwrap_or_else(|_| DEFAULT_DOMAIN.to_string()),
            api_addr: env::var("API_ADDR").unwrap_or_else(|_| DEFAULT_API_ADDR.to_string()),
            ssh_addr: env::var("SSH_ADDR").unwrap_or_else(|_| DEFAULT_SSH_ADDR.to_string()),
            max_tunnels_per_ip: env::var("MAX_TUNNELS_PER_IP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_MAX_TUNNELS_PER_IP),
            global_tunnel_limit: env::var("GLOBAL_TUNNEL_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_GLOBAL_TUNNEL_LIMIT),
        };

        info!(
            api = %config.api_addr,
            ssh = %config.ssh_addr,
            domain = %config.domain,
            max_per_ip = config.max_tunnels_per_ip,
            global_limit = config.global_tunnel_limit,
            "loaded configuration"
        );

        config
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!(
            "{key} is required but not set. check your .env file or environment variables."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn defaults_are_sane() {
        assert_eq!(DEFAULT_API_ADDR, "0.0.0.0:3000");
        assert_eq!(DEFAULT_SSH_ADDR, "0.0.0.0:2222");
        assert_eq!(DEFAULT_MAX_TUNNELS_PER_IP, 5);
        assert_eq!(DEFAULT_GLOBAL_TUNNEL_LIMIT, 1000);
    }
}
