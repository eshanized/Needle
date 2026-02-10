// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use lazy_static::lazy_static;
use prometheus::{
    CounterVec, Gauge, HistogramVec, register_counter_vec, register_gauge, register_histogram_vec,
};

lazy_static! {
    /// Gauge tracking current number of active tunnels
    pub static ref TUNNELS_ACTIVE: Gauge = register_gauge!(
        "needle_tunnels_active",
        "Current number of active tunnels"
    )
    .expect("failed to register needle_tunnels_active metric");

    /// Counter tracking total tunnels created since server start
    pub static ref TUNNELS_CREATED: CounterVec = register_counter_vec!(
        "needle_tunnels_created_total",
        "Total number of tunnels created",
        &["protocol"]
    )
    .expect("failed to register needle_tunnels_created_total metric");

    /// Counter tracking total tunnels destroyed since server start
    pub static ref TUNNELS_DESTROYED: CounterVec = register_counter_vec!(
        "needle_tunnels_destroyed_total",
        "Total number of tunnels destroyed",
        &["reason"]
    )
    .expect("failed to register needle_tunnels_destroyed_total metric");

    /// Counter tracking authentication failures by type (ssh, api)
    pub static ref AUTH_FAILURES: CounterVec = register_counter_vec!(
        "needle_auth_failures_total",
        "Total number of authentication failures",
        &["auth_type", "reason"]
    )
    .expect("failed to register needle_auth_failures_total metric");

    /// Histogram tracking HTTP request latency through tunnels
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "needle_http_request_duration_seconds",
        "HTTP request duration through tunnels in seconds",
        &["method", "status_code"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]
    )
    .expect("failed to register needle_http_request_duration_seconds metric");

    /// Counter tracking errors by type
    pub static ref ERRORS: CounterVec = register_counter_vec!(
        "needle_errors_total",
        "Total number of errors",
        &["error_type"]
    )
    .expect("failed to register needle_errors_total metric");

    /// Counter tracking rate limit hits
    pub static ref RATE_LIMIT_HITS: CounterVec = register_counter_vec!(
        "needle_rate_limit_hits_total",
        "Total number of rate limit hits",
        &["limit_type"]
    )
    .expect("failed to register needle_rate_limit_hits_total metric");
}

/// Increment tunnel creation counter
pub fn tunnel_created(protocol: &str) {
    TUNNELS_CREATED.with_label_values(&[protocol]).inc();
    TUNNELS_ACTIVE.inc();
}

/// Increment tunnel destruction counter
pub fn tunnel_destroyed(reason: &str) {
    TUNNELS_DESTROYED.with_label_values(&[reason]).inc();
    TUNNELS_ACTIVE.dec();
}

/// Increment auth failure counter
pub fn auth_failure(auth_type: &str, reason: &str) {
    AUTH_FAILURES.with_label_values(&[auth_type, reason]).inc();
}

/// Record HTTP request duration
pub fn http_request_duration(method: &str, status_code: u16, duration_secs: f64) {
    HTTP_REQUEST_DURATION
        .with_label_values(&[method, &status_code.to_string()])
        .observe(duration_secs);
}

/// Increment error counter
pub fn error_occurred(error_type: &str) {
    ERRORS.with_label_values(&[error_type]).inc();
}

/// Increment rate limit hit counter
pub fn rate_limit_hit(limit_type: &str) {
    RATE_LIMIT_HITS.with_label_values(&[limit_type]).inc();
}
