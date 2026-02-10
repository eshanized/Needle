# Testing

Testing strategy and guidelines for Needle.

## Testing Philosophy

- **Units tests** for business logic and utilities
- **Integration tests** for API endpoints and databases
- **No mocking** where possible - use real databases (Supabase test project)
- **Fast feedback** - Unit tests run in milliseconds

## Running Tests

### All Tests
```bash
cargo test --workspace
```

### Specific Crate
```bash
cargo test -p needle-core
cargo test -p needle-api
```

### Specific Test
```bash
cargo test test_subdomain_validation
```

### With Output
```bash
cargo test -- --nocapture
```

## Unit Tests

### Location

Place tests in the same file as the code being tested:

```rust
// needle-common/src/subdomain.rs

pub fn is_valid_custom(subdomain: &str) -> bool {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_subdomains_pass() {
        assert!(is_valid_custom("my-app"));
        assert!(is_valid_custom("staging-v2"));
    }

    #[test]
    fn invalid_subdomains_fail() {
        assert!(!is_valid_custom("_invalid"));
        assert!(!is_valid_custom("my_app"));
        assert!(!is_valid_custom("ab"));  // too short
    }
}
```

### What to Test

- **Edge cases** - Empty strings, null, boundary values
- **Error paths** - Invalid input, failed operations
- **Business rules** - Tier limits, subdomain format, etc.

### Example: Configuration Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_hierarchy_is_enforced() {
        let config = NeedleConfig {
            free_tier_limit: 10,
            pro_tier_limit: 5,  // Invalid: should be > free
            // ...
        };
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn invalid_domain_rejected() {
        let config = NeedleConfig {
            domain: "..invalid".to_string(),
            // ...
        };
        
        assert!(config.validate().is_err());
    }
}
```

## Integration Tests

### Location

Place in `tests/` directory (workspace root or crate root):

```
libneedle/
└── tests/
    ├── api_auth.rs
    ├── tunnel_lifecycle.rs
    └── common/
        └── mod.rs  # Shared test utilities
```

### Example: API Test

```rust
// tests/api_auth.rs

use needle_api::create_router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;  // For oneshot()

#[tokio::test]
async fn test_login_with_valid_credentials() {
    let app = create_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                    "email": "test@example.com",
                    "password": "password123"
                }"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let app = create_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(r#"{
                    "email": "test@example.com",
                    "password": "wrong"
                }"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### Test Database Setup

Use a separate Supabase project for testing:

```bash
# .env.test
SUPABASE_URL=https://test-project.supabase.co
SUPABASE_ANON_KEY=test-anon-key
SUPABASE_SERVICE_ROLE_KEY=test-service-key
JWT_SECRET=test-secret
```

Load in tests:

```rust
#[tokio::test]
async fn test_with_database() {
    dotenvy::from_filename(".env.test").ok();
    let config = NeedleConfig::from_env();
    
    let db = SupabaseClient::new(&config);
    
    // Run test...
}
```

## Test Utilities

### Common Module

Share test utilities:

```rust
// tests/common/mod.rs

pub fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        tier: "free".to_string(),
        // ...
    }
}

pub async fn cleanup_test_data(db: &SupabaseClient) {
    // Delete test users, tunnels, etc.
}
```

Use in tests:

```rust
mod common;

#[tokio::test]
async fn my_test() {
    let user = common::create_test_user();
    // ...
}
```

## Testing Patterns

### Testing Errors

```rust
#[test]
fn test_subdomain_too_short() {
    let result = validate_subdomain("ab");
    
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "subdomain must be at least 3 characters"
    );
}
```

### Testing Async Functions

```rust
#[tokio::test]
async fn test_create_tunnel() {
    let manager = TunnelManager::new(/* ... */);
    
    let result = manager.create_tunnel(
        user_id,
        Some("my-app".to_string()),
        3000,
    ).await;
    
    assert!(result.is_ok());
    let tunnel = result.unwrap();
    assert_eq!(tunnel.subdomain, "my-app");
}
```

### Parameterized Tests

```rust
#[test]
fn test_tier_limits() {
    let test_cases = vec![
        ("free", 3),
        ("pro", 50),
        ("enterprise", 500),
    ];
    
    let config = NeedleConfig::from_env();
    
    for (tier, expected_limit) in test_cases {
        assert_eq!(config.tier_limit(tier), expected_limit);
    }
}
```

## Coverage

### Generate Coverage Report

Install `tarpaulin`:

```bash
cargo install cargo-tarpaulin
```

Run:

```bash
cargo tarpaulin --out Html
```

Open `tarpaulin-report.html`

### Coverage Goals

- **Critical paths**: 100% (auth, tunnel creation, payment logic)
- **Business logic**: 80%+
- **Utilities**: 70%+
- **Overall**: 60%+

## Continuous Integration

Tests run on every PR via GitHub Actions:

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --workspace
      - run: cargo clippy --all -- -D warnings
      - run: cargo fmt --all -- --check
```

## Best Practices

✅ **DO**:
- Test edge cases and error paths
- Use descriptive test names (`test_subdomain_collision_returns_error`)
- Keep tests independent (no shared state)
- Use `.unwrap()` freely in tests
- Test one thing per test

❌ **DON'T**:
- Mock unless absolutely necessary
- Test implementation details
- Write tests that depend on execution order
- Commit tests that fail intermittently
- Skip writing tests for bug fixes

## Next Steps

- [Contributing](./contributing.md) - Development workflow
- [API Reference](./api-reference.md) - Endpoints to test
