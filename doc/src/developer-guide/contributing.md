# Contributing

Guidelines for contributing to Needle.

## Coding Standards

### File Headers

Every source file **must** include this header:

```rust
// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT
```

### Comments

- ✅ **Complex functions** - Add explanatory comments
- ✅ **Natural tone** - Write for humans, not robots
- ❌ **Obvious comments** - Don't comment self-explanatory code
- ❌ **Redundant inline comments** - Code should be self-documenting

Example:

```rust
// ✅ GOOD - Explains non-obvious behavior
// Retry with exponential backoff because Supabase may temporarily throttle
let result = retry_with_backoff(|| db.query()).await?;

// ❌ BAD - States the obvious
// Increment counter
counter += 1;
```

### Code Quality

- **No hardcoded values** - Use config constants or environment variables
- **All configurable values** in `config.rs` or `NeedleConfig`
- **Error handling** - Use `?` operator, never `.unwrap()` in production code
- **Descriptive naming** - `create_tunnel()` not `ct()`

### UI Design (needleui)

**Theme**: Dark Everblush

```css
--bg: #141b1e;
--surface: #232a2d;
--red: #e57474;
--green: #8ccf7e;
--yellow: #e5c76b;
--blue: #67b0e8;
--magenta: #c47fd5;
--cyan: #6cbfbf;
--gray: #b3b9b8;
--fg: #dadada;
```

**Icons**: SVG only, no emoji in UI  
**Styling**: Vanilla CSS, no Tailwind

## Git Workflow

### Conventional Commits

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]
```

**Types**:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

**Format**: All lowercase, no period at end

Examples:
```
feat: add websocket proxy support
fix: prevent subdomain collision race condition
docs: add api reference for tunnel endpoints
refactor: extract ssh handler into separate module
test: add integration tests for tunnel lifecycle
```

### Branch Strategy

- `main` - Production-ready code
- `develop` - Integration branch
- `feat/feature-name` - New features
- `fix/bug-description` - Bug fixes

### Pull Request Process

1. **Create feature branch** from `develop`
2. **Make changes** following coding standards
3. **Write tests** for new functionality
4. **Run tests** - `cargo test --workspace`
5. **Run formatter** - `cargo fmt --all`
6. **Run linter** - `cargo clippy --all -- -D warnings`
7. **Update docs** if API changed
8. **Commit** with conventional commits
9. **Push** and create PR
10. **Request review** from maintainers
11. **Address feedback** and update PR
12. **Merge** after approval

## Development Setup

### Backend

```bash
cd libneedle
cp .env.example .env
# Edit .env with your Supabase credentials
cargo build
cargo test
cargo run
```

### Frontend

```bash
cd needleui
npm install
npm run dev
```

### Running Both

Terminal 1:
```bash
cd libneedle && cargo run
```

Terminal 2:
```bash
cd needleui && npm run dev
```

## Testing

### Unit Tests

Place tests in same file as code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subdomain_validation() {
        assert!(is_valid_custom("my-app"));
        assert!(!is_valid_custom("_invalid"));
    }
}
```

Run: `cargo test`

### Integration Tests

Place in `tests/` directory:

```bash
libneedle/tests/
└── tunnel_lifecycle.rs
```

Run: `cargo test --test tunnel_lifecycle`

## Documentation

### Rust Docs

Add doc comments to public APIs:

```rust
/// Creates a new tunnel for the given user.
///
/// # Arguments
///
/// * `user_id` - The user's UUID
/// * `subdomain` - Optional custom subdomain
///
/// # Returns
///
/// Returns `Ok(Tunnel)` on success or `Err(NeedleError)` if
/// subdomain is already taken or user has hit tier limit.
pub async fn create_tunnel(
    user_id: Uuid,
    subdomain: Option<String>,
) -> Result<Tunnel, NeedleError> {
    // ...
}
```

Generate docs: `cargo doc --open`

### mdBook Docs

Edit files in `doc/src/`:

```bash
cd doc
mdbook serve --open
```

## Common Tasks

### Add a New API Endpoint

1. Create handler in `needle-api/src/routes/`
2. Add route to router in `needle-api/src/lib.rs`
3. Add tests in same file
4. Update `doc/src/developer-guide/api-reference.md`
5. Commit: `feat: add endpoint for XYZ`

### Add a New Configuration Option

1. Add field to `NeedleConfig` in `needle-core/src/config.rs`
2. Add default constant
3. Add validation in `validate()`
4. Update `.env.example`
5. Update `doc/src/user-guide/configuration.md`
6. Commit: `feat: add config option for XYZ`

### Fix a Bug

1. Write a failing test that reproduces the bug
2. Fix the bug
3. Verify test now passes
4. Commit: `fix: correct XYZ behavior when ...`

## Code Review Checklist

Reviewers should check:

- [ ] Follows coding standards (headers, comments, naming)
- [ ] No hardcoded values
- [ ] Proper error handling (no `.unwrap()` in prod code)
- [ ] Tests added for new functionality
- [ ] Documentation updated if API changed
- [ ] Conventional commit messages
- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] All tests pass

## Need Help?

- **Questions**: Open a GitHub discussion
- **Bugs**: Open an issue with reproduction steps
- **Features**: Open an issue describing the use case

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
