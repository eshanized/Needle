// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use crate::metrics;
use needle_common::error::{NeedleError, Result};
use needle_common::rate_limit::RateLimiter;
use needle_common::subdomain;
use needle_db::client::SupabaseClient;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct ActiveTunnel {
    pub subdomain: String,
    pub listener: TcpListener,
    pub bind_addr: SocketAddr,
    pub client_ip: String,
    pub user_id: Uuid,
    pub rate_limiter: RateLimiter,
}

/// Keeps track of all live tunnels on this server instance. Each tunnel
/// has a subdomain that maps to an internal TCP listener, which receives
/// proxied HTTP traffic and forwards it back through the SSH channel to
/// the user's local app.
///
/// The manager enforces capacity limits (per-IP and global), handles
/// subdomain uniqueness, and cleans up resources when tunnels close.
pub struct TunnelManager {
    tunnels: HashMap<String, Arc<ActiveTunnel>>,
    ip_counts: HashMap<String, usize>,
    db: SupabaseClient,
    max_tunnels_per_ip: usize,
    global_tunnel_limit: usize,
    requests_per_second: f64,
    burst_size: f64,
}

impl TunnelManager {
    pub fn new(
        db: SupabaseClient,
        max_tunnels_per_ip: usize,
        global_tunnel_limit: usize,
        requests_per_second: f64,
        burst_size: f64,
    ) -> Self {
        Self {
            tunnels: HashMap::new(),
            ip_counts: HashMap::new(),
            db,
            max_tunnels_per_ip,
            global_tunnel_limit,
            requests_per_second,
            burst_size,
        }
    }

    /// Spins up a new tunnel by optionally using a custom subdomain or generating
    /// a unique one, binding a local TCP listener, and registering everything in
    /// both the in-memory map and the database.
    ///
    /// The flow goes: query user tier → check tier limit → validate subdomain →
    /// check IP/global limits → bind listener → save to database → register in memory.
    /// If any step fails, we bail out early without leaving orphaned state.
    pub async fn create(
        &mut self,
        client_ip: &str,
        user_id: Uuid,
        custom_subdomain: Option<String>,
        target_port: i32,
        protocol: &str,
        is_persistent: bool,
    ) -> Result<Arc<ActiveTunnel>> {
        // Count existing tunnels for this user to enforce per-user limits
        let user_tunnel_count = self.tunnels.values()
            .filter(|t| t.user_id == user_id)
            .count();

        // Apply tier enforcement using reasonable defaults
        // TODO: Query actual user tier from database once queries::users::get_tier implemented
        // For now, enforce a reasonable per-user limit
        const DEFAULT_USER_TUNNEL_LIMIT: usize = 10;
        
        if user_tunnel_count >= DEFAULT_USER_TUNNEL_LIMIT {
            warn!(
                user_id = %user_id,
                current = user_tunnel_count,
                limit = DEFAULT_USER_TUNNEL_LIMIT,
                "per-user tunnel limit exceeded"
            );
            metrics::error_occurred("tier_limit_exceeded");
            return Err(NeedleError::ServerAtCapacity); // Temporary - use proper tier error once tier lookup works
        }

        // Check IP-based limit
        let ip_count = self.ip_counts.get(client_ip).copied().unwrap_or(0);
        if ip_count >= self.max_tunnels_per_ip {
            return Err(NeedleError::MaxTunnelsPerIp);
        }

        // Check global capacity
        if self.tunnels.len() >= self.global_tunnel_limit {
            return Err(NeedleError::ServerAtCapacity);
        }

        let sub = if let Some(custom) = custom_subdomain {
            // Use custom subdomain if provided
            if self.tunnels.contains_key(&custom) {
                return Err(NeedleError::SubdomainTaken(custom));
            }
            custom
        } else {
            // Generate unique subdomain
            self.generate_unique_subdomain()?
        };

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let bind_addr = listener.local_addr()?;

        info!(subdomain = %sub, addr = %bind_addr, "tunnel created");

        // Attempt database write, rollback listener on failure
        match needle_db::queries::tunnels::create(
            &self.db,
            &user_id.to_string(),
            &sub,
            target_port,
            protocol,
            is_persistent,
        )
        .await
        {
            Ok(_) => {},
            Err(e) => {
                // Rollback: close listener and return error
                drop(listener);
                metrics::error_occurred("tunnel_db_write_failed");
                return Err(e);
            }
        }

        let tunnel = Arc::new(ActiveTunnel {
            subdomain: sub.clone(),
            listener,
            bind_addr,
            client_ip: client_ip.to_string(),
            user_id,
            rate_limiter: RateLimiter::new(self.requests_per_second, self.burst_size),
        });

        self.tunnels.insert(sub.clone(), tunnel.clone());
        *self.ip_counts.entry(client_ip.to_string()).or_insert(0) += 1;

        // Record metrics
        metrics::tunnel_created(protocol);

        Ok(tunnel)
    }

    pub fn get(&self, subdomain: &str) -> Option<Arc<ActiveTunnel>> {
        self.tunnels.get(subdomain).cloned()
    }

    pub async fn remove(&mut self, sub: &str) -> Result<()> {
        if let Some(tunnel) = self.tunnels.remove(sub) {
            if let Some(count) = self.ip_counts.get_mut(&tunnel.client_ip) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    self.ip_counts.remove(&tunnel.client_ip);
                }
            }

            needle_db::queries::tunnels::set_active(&self.db, sub, false).await?;
            info!(subdomain = %sub, "tunnel removed");

            // Record metrics
            metrics::tunnel_destroyed("user_deleted");
        }

        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.tunnels.len()
    }

    pub fn tunnels_for_ip(&self, ip: &str) -> usize {
        self.ip_counts.get(ip).copied().unwrap_or(0)
    }

    fn generate_unique_subdomain(&self) -> Result<String> {
        for _ in 0..10 {
            let sub = subdomain::generate();
            if !self.tunnels.contains_key(&sub) {
                return Ok(sub);
            }
            warn!("subdomain collision, retrying");
        }
        Err(NeedleError::ServerAtCapacity)
    }

    pub fn db_client(&self) -> &SupabaseClient {
        &self.db
    }
}
