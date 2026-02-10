// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use prometheus::{Encoder, TextEncoder};

/// Prometheus metrics endpoint handler
/// Returns metrics in Prometheus text format
pub async fn metrics() -> Response {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = vec![];
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => {
            // Return metrics with proper content type
            (
                StatusCode::OK,
                [("content-type", encoder.format_type())],
                buffer,
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to encode metrics");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to encode metrics",
            )
                .into_response()
        }
    }
}
