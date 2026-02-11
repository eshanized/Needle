//! Integration tests for Needle production certification
//!
//! These tests validate end-to-end functionality including:
//! - Tunnel creation and lifecycle
//! - Authentication flows  
//! - Tier enforcement
//! - Configuration validation
//! - Metrics endpoint

use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

const BASE_URL: &str = "http://localhost:3000";
const TEST_EMAIL: &str = "test@example.com";
const TEST_PASSWORD: &str = "securepassword123";

/// Helper to create auth token for tests
async fn get_auth_token(client: &Client) -> Result<String, Box<dyn std::error::Error>> {
    // Register user
    let register_response = client
        .post(&format!("{}/api/auth/register", BASE_URL))
        .json(&json!({
            "email": TEST_EMAIL,
            "username": "testuser",
            "password": TEST_PASSWORD
        }))
        .send()
        .await?;

    if !register_response.status().is_success() {
        // User might already exist, try login instead
        let login_response = client
            .post(&format!("{}/api/auth/login", BASE_URL))
            .json(&json!({
                "email": TEST_EMAIL,
                "password": TEST_PASSWORD
            }))
            .send()
            .await?;

        let login_data: Value = login_response.json().await?;
        return Ok(login_data["token"].as_str().unwrap().to_string());
    }

    let register_data: Value = register_response.json().await?;
    Ok(register_data["token"].as_str().unwrap().to_string())
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_health_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let response = client
        .get(&format!("{}/health", BASE_URL))
        .send()
        .await?;

    assert!(response.status().is_success());
    
    let body: Value = response.json().await?;
    assert_eq!(body["status"], "ok");
    
    Ok(())
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_metrics_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let response = client
        .get(&format!("{}/metrics", BASE_URL))
        .send()
        .await?;

    assert!(response.status().is_success());
    
    let body = response.text().await?;
    
    // Verify Prometheus format
    assert!(body.contains("needle_tunnels_active"));
    assert!(body.contains("needle_tunnels_created_total"));
    assert!(body.contains("needle_auth_failures_total"));
    
    Ok(())
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_auth_flow() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // 1. Register
    let register_response = client
        .post(&format!("{}/api/auth/register", BASE_URL))
        .json(&json!({
            "email": format!("test_{}@example.com", chrono::Utc::now().timestamp()),
            "username": format!("user_{}", chrono::Utc::now().timestamp()),
            "password": TEST_PASSWORD
        }))
        .send()
        .await?;

    assert!(register_response.status().is_success());
    let register_data: Value = register_response.json().await?;
    assert!(register_data["token"].is_string());
    let token = register_data["token"].as_str().unwrap();

    // 2. Use token to access protected endpoint
    let tunnels_response = client
        .get(&format!("{}/api/tunnels", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(tunnels_response.status().is_success());

    // 3. Revoke token
    let revoke_response = client
        .post(&format!("{}/api/auth/revoke", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(revoke_response.status().is_success());

    // 4. Try to use revoked token - should fail
    let after_revoke_response = client
        .get(&format!("{}/api/tunnels", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert_eq!(after_revoke_response.status(), 401);
    
    Ok(())
}

#[tokio::test]
#[ignore] // Requires running server  
async fn test_tunnel_creation() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let token = get_auth_token(&client).await?;

    // Create a tunnel
    let create_response = client
        .post(&format!("{}/api/tunnels", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "target_port": 8080,
            "protocol": "http"
        }))
        .send()
        .await?;

    assert!(create_response.status().is_success());
    
    let tunnel_data: Value = create_response.json().await?;
    assert!(tunnel_data["subdomain"].is_string());
    assert!(tunnel_data["url"].is_string());
    
    let subdomain = tunnel_data["subdomain"].as_str().unwrap();

    // List tunnels
    let list_response = client
        .get(&format!("{}/api/tunnels", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(list_response.status().is_success());
    let tunnels: Value = list_response.json().await?;
    let tunnels_array = tunnels.as_array().unwrap();
    assert!(tunnels_array.iter().any(|t| t["subdomain"] == subdomain));

    // Delete tunnel
    let delete_response = client
        .delete(&format!("{}/api/tunnels/{}", BASE_URL, subdomain))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(delete_response.status().is_success());

    Ok(())
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_tier_limit_enforcement() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let token = get_auth_token(&client).await?;

    // Create tunnels up to limit (default user tunnel limit is 10)
    let mut created_tunnels = Vec::new();
    
    for i in 0..11 {
        let create_response = client
            .post(&format!("{}/api/tunnels", BASE_URL))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({
                "target_port": 8080 + i,
                "protocol": "http"
            }))
            .send()
            .await?;

        if i < 10 {
            // Should succeed for first 10
            assert!(create_response.status().is_success());
            if let Ok(data) = create_response.json::<Value>().await {
                created_tunnels.push(data["subdomain"].as_str().unwrap().to_string());
            }
        } else {
            // 11th should fail (tier limit exceeded)
            assert!(!create_response.status().is_success());
        }
    }

    // Cleanup
    for subdomain in created_tunnels {
        client
            .delete(&format!("{}/api/tunnels/{}", BASE_URL, subdomain))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_invalid_auth() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Try to access protected endpoint without token
    let no_auth_response = client
        .get(&format!("{}/api/tunnels", BASE_URL))
        .send()
        .await?;

    assert_eq!(no_auth_response.status(), 401);

    // Try with invalid token
    let invalid_token_response = client
        .get(&format!("{}/api/tunnels", BASE_URL))
        .header("Authorization", "Bearer invalid_token_here")
        .send()
        .await?;

    assert_eq!(invalid_token_response.status(), 401);

    Ok(())
}

/// Test configuration validation by checking startup logs
#[test]
fn test_config_validation_rules() {
    use needle_core::config::NeedleConfig;
    use std::env;

    // Save original env
    let original_supabase_url = env::var("SUPABASE_URL").ok();
    
    // Set minimal required env vars
    env::set_var("SUPABASE_URL", "https://test.supabase.co");
    env::set_var("SUPABASE_ANON_KEY", "test_anon_key");
    env::set_var("SUPABASE_SERVICE_ROLE_KEY", "test_service_key");
    env::set_var("JWT_SECRET", "test_secret_at_least_32_chars_long_for_security");

    // Test invalid domain
    env::set_var("DOMAIN", "..invalid");
    let result = std::panic::catch_unwind(|| {
        NeedleConfig::from_env()
    });
    assert!(result.is_err());

    // Test invalid timeout
    env::set_var("DOMAIN", "valid.com");
    env::set_var("HTTP_READ_TIMEOUT_SECS", "0");
    let result = std::panic::catch_unwind(|| {
        NeedleConfig::from_env()
    });
    assert!(result.is_err());

    // Restore original env
    if let Some(url) = original_supabase_url {
        env::set_var("SUPABASE_URL", url);
    } else {
        env::remove_var("SUPABASE_URL");
    }
}

#[cfg(test)]
mod readme {
    //! # Running Integration Tests
    //!
    //! These tests require a running Needle server instance.
    //!
    //! ## Setup
    //!
    //! 1. Start the server:
    //! ```bash
    //! cargo run --bin needle-server
    //! ```
    //!
    //! 2. In another terminal, run tests:
    //! ```bash
    //! cargo test --test integration -- --ignored --test-threads=1
    //! ```
    //!
    //! ## Test Coverage
    //!
    //! - ✅ Health endpoint
    //! - ✅ Metrics endpoint (Prometheus format)
    //! - ✅ Auth flow (register → login → token → revoke)
    //! - ✅ Tunnel lifecycle (create → list → delete)
    //! - ✅ Tier limit enforcement
    //! - ✅ Invalid auth rejection
    //! - ✅ Configuration validation
}
