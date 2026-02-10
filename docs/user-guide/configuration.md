# Configuration

Complete reference for all Needle configuration options via environment variables.

## Configuration Philosophy

Needle follows the [Twelve-Factor App](https://12factor.net/) methodology:
- All configuration via **environment variables**
- **No config files** to manage
- **Sensible defaults** for everything except secrets
- **Fail fast** on startup if required values are missing

## Required Variables

These variables **MUST** be set or the server will refuse to start:

### `SUPABASE_URL`
- **Type**: URL
- **Example**: `https://xxxxx.supabase.co`
- **Description**: Your Supabase project URL
- **Where to find**: Supabase Dashboard → Settings → API → Project URL

### `SUPABASE_ANON_KEY`
- **Type**: JWT string
- **Example**: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`
- **Description**: Supabase anonymous key for client-side auth
- **Where to find**: Supabase Dashboard → Settings → API → anon/public

### `SUPABASE_SERVICE_ROLE_KEY`
- **Type**: JWT string
- **Example**: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`
- **Description**: Supabase service role key (bypasses RLS)
- **Where to find**: Supabase Dashboard → Settings → API → service_role
- **⚠️ CRITICAL**: This key grants full database access - keep it secret!

### `JWT_SECRET`
- **Type**: Hex string (64 characters minimum)
- **Example**: `a1b2c3d4e5f6...` (64+ hex chars)
- **Description**: Secret key for signing JWT tokens
- **Generate**: `openssl rand -hex 32`
- **⚠️ CRITICAL**: If this leaks, all JWT tokens are compromised!

## Server Configuration

### `API_ADDR`
- **Type**: `host:port` socket address
- **Default**: `0.0.0.0:3000`
- **Example**: `127.0.0.1:8080`
- **Description**: Address for the HTTP API server
- **Notes**: 
  - Use `0.0.0.0` to listen on all interfaces
  - Use `127.0.0.1` for localhost only

### `SSH_ADDR`
- **Type**: `host:port` socket address
- **Default**: `0.0.0.0:2222`
- **Example**: `0.0.0.0:2222`
- **Description**: Address for the SSH tunnel server
- **Notes**:
  - Port 22 requires root - use 2222 instead
  - Must be publicly accessible for tunnels to work

### `DOMAIN`
- **Type**: Domain name string
- **Default**: `localhost`
- **Example**: `tunnel.example.com`
- **Description**: Base domain for tunnel subdomains
- **Notes**:
  - Must have wildcard DNS configured (`*.yourdomain.com`)
  - Used to construct tunnel URLs like `xyz.yourdomain.com`

## Tunnel Limits

### `MAX_TUNNELS_PER_IP`
- **Type**: Positive integer
- **Default**: `5`
- **Range**: `1` to `1000`
- **Description**: Maximum concurrent tunnels from a single IP address
- **Use case**: Prevent single users from monopolizing resources

### `GLOBAL_TUNNEL_LIMIT`
- **Type**: Positive integer
- **Default**: `1000`
- **Range**: `100` to `100000`
- **Description**: Maximum total concurrent tunnels across all users
- **Use case**: Protect server resources

## HTTP Proxy Configuration

### `HTTP_READ_TIMEOUT_SECS`
- **Type**: Positive integer (seconds)
- **Default**: `10`
- **Range**: `1` to `300`
- **Description**: Maximum time to wait for reading HTTP requests
- **Use case**: Prevent slow clients from holding connections

### `HTTP_WRITE_TIMEOUT_SECS`
- **Type**: Positive integer (seconds)
- **Default**: `10`
- **Range**: `1` to `300`
- **Description**: Maximum time to wait for writing HTTP responses
- **Use case**: Prevent slow backends from blocking

## Tier Limits

### `FREE_TIER_LIMIT`
- **Type**: Positive integer
- **Default**: `3`
- **Description**: Maximum concurrent tunnels for free-tier users

### `PRO_TIER_LIMIT`
- **Type**: Positive integer
- **Default**: `50`
- **Description**: Maximum concurrent tunnels for pro-tier users
- **Must be**: Greater than `FREE_TIER_LIMIT`

### `ENTERPRISE_TIER_LIMIT`
- **Type**: Positive integer
- **Default**: `500`
- **Description**: Maximum concurrent tunnels for enterprise-tier users
- **Must be**: Greater than `PRO_TIER_LIMIT`

## SSH Security

### `MIN_SSH_PORT`
- **Type**: Port number (1024-65535)
- **Default**: `1024`
- **Description**: Minimum allowed port for SSH port forwarding
- **Use case**: Prevent users from forwarding privileged ports (< 1024)
- **Note**: Must be >= 1024

## Logging

### `RUST_LOG`
- **Type**: Log filter string
- **Default**: (none - shows only errors)
- **Examples**:
  - `needle=info` - Info level for Needle modules
  - `needle=debug,tower_http=info` - Debug Needle, info Tower
  - `debug` - Debug everything (very verbose!)
- **Description**: Controls log verbosity using `tracing-subscriber` syntax
- **Levels**: `error`, `warn`, `info`, `debug`, `trace`

## CORS Configuration

### `CORS_ORIGIN`
- **Type**: URL or `*`
- **Default**: `*` (allow all origins)
- **Example**: `https://dashboard.yourdomain.com`
- **Description**: Allowed CORS origins for the API
- **Security**: Set to your dashboard URL in production

## Configuration Validation

On startup, Needle validates all configuration:

| Check | Validation |
|-------|-----------|
| Domain format | No empty, no `..`, no leading `.` |
| Socket addresses | Must parse as valid `SocketAddr` |
| Tier hierarchy | Pro > Free, Enterprise > Pro |
| Port range | `MIN_SSH_PORT` >= 1024 |
| Timeouts | > 0 seconds, warning if > 300 |
| Limits | All limits > 0 |

**If validation fails**, the server panics with a clear error message.

## Example Configurations

### Development (localhost)

```bash
# .env for local development
SUPABASE_URL=https://xxxxx.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-key
JWT_SECRET=$(openssl rand -hex 32)
DOMAIN=localhost
API_ADDR=127.0.0.1:3000
SSH_ADDR=127.0.0.1:2222
RUST_LOG=needle=debug,tower_http=debug
FREE_TIER_LIMIT=10
MAX_TUNNELS_PER_IP=10
```

### Staging

```bash
# .env for staging server
SUPABASE_URL=https://staging-xxxxx.supabase.co
SUPABASE_ANON_KEY=staging-anon-key
SUPABASE_SERVICE_ROLE_KEY=staging-service-key
JWT_SECRET=$(openssl rand -hex 32)
DOMAIN=staging.yourdomain.com
API_ADDR=0.0.0.0:3000
SSH_ADDR=0.0.0.0:2222
RUST_LOG=needle=info,tower_http=info
CORS_ORIGIN=https://dashboard-staging.yourdomain.com
FREE_TIER_LIMIT=5
PRO_TIER_LIMIT=100
GLOBAL_TUNNEL_LIMIT=2000
HTTP_READ_TIMEOUT_SECS=15
HTTP_WRITE_TIMEOUT_SECS=15
```

### Production

```bash
# .env for production
SUPABASE_URL=https://prod-xxxxx.supabase.co
SUPABASE_ANON_KEY=prod-anon-key
SUPABASE_SERVICE_ROLE_KEY=prod-service-key
JWT_SECRET=$(openssl rand -hex 32)
DOMAIN=tunnel.yourdomain.com
API_ADDR=0.0.0.0:3000
SSH_ADDR=0.0.0.0:2222
RUST_LOG=needle=info,tower_http=warn
CORS_ORIGIN=https://dashboard.yourdomain.com
FREE_TIER_LIMIT=3
PRO_TIER_LIMIT=50
ENTERPRISE_TIER_LIMIT=500
GLOBAL_TUNNEL_LIMIT=5000
MAX_TUNNELS_PER_IP=5
HTTP_READ_TIMEOUT_SECS=10
HTTP_WRITE_TIMEOUT_SECS=10
MIN_SSH_PORT=1024
```

## Security Best Practices

> [!CAUTION]
> **Never commit `.env` files to version control!**

### 1. Generate Strong Secrets

```bash
# Good: Cryptographically random
JWT_SECRET=$(openssl rand -hex 32)

# Bad: Weak, predictable
JWT_SECRET=mysecret123
```

### 2. Rotate Secrets Regularly

- Rotate `JWT_SECRET` monthly (invalidates all tokens)
- Rotate Supabase keys if compromised
- Use different secrets for dev/staging/prod

### 3. Restrict CORS in Production

```bash
# Development: Allow all
CORS_ORIGIN=*

# Production: Specific domain only
CORS_ORIGIN=https://dashboard.yourdomain.com
```

### 4. Use Environment-Specific Configs

Never reuse production credentials in development!

### 5. Limit Exposure

```bash
# Good: Localhost only for internal APIs
API_ADDR=127.0.0.1:3000

# Good: Public for tunnel endpoints
SSH_ADDR=0.0.0.0:2222
```

## Runtime Configuration Changes

> [!NOTE]
> Configuration changes require a server restart.

To apply new configuration:

```bash
# 1. Edit .env file
nano .env

# 2. Restart the server
systemctl restart needle

# 3. Verify new config was loaded
journalctl -u needle -n 20 | grep "loaded configuration"
```

## Next Steps

- [Dashboard](./dashboard.md) - Managing configuration via web UI
- [Deployment](../operations/deployment.md) - Production configuration
- [Security](../operations/security.md) - Hardening your configuration
