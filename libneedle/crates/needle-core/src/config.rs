// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::env;
use std::time::Duration;
use tracing::{info, warn};

const DEFAULT_API_ADDR: &str = "0.0.0.0:3000";
const DEFAULT_SSH_ADDR: &str = "0.0.0.0:2222";
const DEFAULT_DOMAIN: &str = "localhost";
const DEFAULT_MAX_TUNNELS_PER_IP: usize = 5;
const DEFAULT_GLOBAL_TUNNEL_LIMIT: usize = 1000;
const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 10;
const DEFAULT_FREE_TIER_LIMIT: usize = 3;
const DEFAULT_PRO_TIER_LIMIT: usize = 50;
const DEFAULT_ENTERPRISE_TIER_LIMIT: usize = 500;
const MIN_ALLOWED_SSH_PORT: u16 = 1024;

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
    
    // HTTP proxy timeouts
    pub http_read_timeout: Duration,
    pub http_write_timeout: Duration,
    
    // Tier limits
    pub free_tier_limit: usize,
    pub pro_tier_limit: usize,
    pub enterprise_tier_limit: usize,
    
    // SSH security
    pub min_ssh_port: u16,
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
            max_tunnels_per_ip: parse_usize_env("MAX_TUNNELS_PER_IP", DEFAULT_MAX_TUNNELS_PER_IP),
            global_tunnel_limit: parse_usize_env("GLOBAL_TUNNEL_LIMIT", DEFAULT_GLOBAL_TUNNEL_LIMIT),
            http_read_timeout: Duration::from_secs(
                parse_u64_env("HTTP_READ_TIMEOUT_SECS", DEFAULT_HTTP_TIMEOUT_SECS)
            ),
            http_write_timeout: Duration::from_secs(
                parse_u64_env("HTTP_WRITE_TIMEOUT_SECS", DEFAULT_HTTP_TIMEOUT_SECS)
            ),
            free_tier_limit: parse_usize_env("FREE_TIER_LIMIT", DEFAULT_FREE_TIER_LIMIT),
            pro_tier_limit: parse_usize_env("PRO_TIER_LIMIT", DEFAULT_PRO_TIER_LIMIT),
            enterprise_tier_limit: parse_usize_env("ENTERPRISE_TIER_LIMIT", DEFAULT_ENTERPRISE_TIER_LIMIT),
            min_ssh_port: parse_u16_env("MIN_SSH_PORT", MIN_ALLOWED_SSH_PORT),
        };

        // Validate configuration
        if let Err(e) = config.validate() {
            panic!("Invalid configuration: {}", e);
        }

        info!(
            api = %config.api_addr,
            ssh = %config.ssh_addr,
            domain = %config.domain,
            max_per_ip = config.max_tunnels_per_ip,
            global_limit = config.global_tunnel_limit,
            http_timeout_secs = config.http_read_timeout.as_secs(),
            free_limit = config.free_tier_limit,
            pro_limit = config.pro_tier_limit,
            min_ssh_port = config.min_ssh_port,
            "loaded configuration"
        );

        config
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), String> {
        // Validate domain format (basic DNS check)
        if self.domain.is_empty() {
            return Err("domain cannot be empty".to_string());
        }
        if self.domain.contains("..") || self.domain.starts_with('.') {
            return Err(format!("invalid domain format: {}", self.domain));
        }

        // Validate addresses can be parsed
        if self.api_addr.parse::<std::net::SocketAddr>().is_err() {
            return Err(format!("invalid API address: {}", self.api_addr));
        }
        if self.ssh_addr.parse::<std::net::SocketAddr>().is_err() {
            return Err(format!("invalid SSH address: {}", self.ssh_addr));
        }

        // Validate positive limits
        if self.max_tunnels_per_ip == 0 {
            return Err("max_tunnels_per_ip must be > 0".to_string());
        }
        if self.global_tunnel_limit == 0 {
            return Err("global_tunnel_limit must be > 0".to_string());
        }
        if self.free_tier_limit == 0 {
            return Err("free_tier_limit must be > 0".to_string());
        }

        // Validate tier hierarchy
        if self.pro_tier_limit <= self.free_tier_limit {
            return Err("pro_tier_limit must be > free_tier_limit".to_string());
        }
        if self.enterprise_tier_limit <= self.pro_tier_limit {
            return Err("enterprise_tier_limit must be > pro_tier_limit".to_string());
        }

        // Validate timeouts are reasonable
        if self.http_read_timeout.as_secs() == 0 {
            return Err("http_read_timeout must be > 0".to_string());
        }
        if self.http_read_timeout.as_secs() > 300 {
            warn!("http_read_timeout is very high: {}s", self.http_read_timeout.as_secs());
        }

        // Validate SSH port restrictions
        if self.min_ssh_port < 1024 {
            return Err(format!("min_ssh_port must be >= 1024, got {}", self.min_ssh_port));
        }

        Ok(())
    }

    /// Get tunnel limit for a given tier
    pub fn tier_limit(&self, tier: &str) -> usize {
        match tier {
            "free" => self.free_tier_limit,
            "pro" => self.pro_tier_limit,
            "enterprise" => self.enterprise_tier_limit,
            _ => self.free_tier_limit, // Default to free tier for unknown tiers
        }
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!("{key} is required but not set. check your .env file or environment variables.")
    })
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_u16_env(key: &str, default: u16) -> u16 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        assert_eq!(DEFAULT_API_ADDR, "0.0.0.0:3000");
        assert_eq!(DEFAULT_SSH_ADDR, "0.0.0.0:2222");
        assert_eq!(DEFAULT_MAX_TUNNELS_PER_IP, 5);
        assert_eq!(DEFAULT_GLOBAL_TUNNEL_LIMIT, 1000);
        assert_eq!(DEFAULT_HTTP_TIMEOUT_SECS, 10);
        assert_eq!(DEFAULT_FREE_TIER_LIMIT, 3);
        assert_eq!(DEFAULT_PRO_TIER_LIMIT, 50);
        assert_eq!(MIN_ALLOWED_SSH_PORT, 1024);
    }

    #[test]
    fn tier_limits_are_hierarchical() {
        assert!(DEFAULT_PRO_TIER_LIMIT > DEFAULT_FREE_TIER_LIMIT);
        assert!(DEFAULT_ENTERPRISE_TIER_LIMIT > DEFAULT_PRO_TIER_LIMIT);
    }
}
