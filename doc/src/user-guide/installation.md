# Installation

This guide covers installing and setting up Needle for development or production use.

## Prerequisites

### Required Software

| Software | Minimum Version | Purpose |
|----------|----------------|---------|
| Rust | 1.80+ | Backend compilation |
| Cargo | 1.80+ | Rust package manager |
| Node.js | 20+ | Frontend build |
| npm | 10+ | Frontend package manager |
| PostgreSQL | 14+ | Database (via Supabase) |

### System Requirements

- **OS**: Linux (recommended), macOS, or Windows with WSL2
- **RAM**: 2GB minimum, 4GB recommended
- **Disk**: 500MB for binaries + database storage
- **Network**: Public IP if hosting for internet access

### Domain Setup

For production use, you'll need:
- A registered domain name
- DNS access to create wildcard A records
- `*.yourdomain.com` pointed to your server IP

## Backend Installation

### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify installation:

```bash
rustc --version  # Should be 1.80 or higher
cargo --version
```

### 2. Clone the Repository

```bash
git clone https://github.com/eshanized/needle.git
cd needle/libneedle
```

### 3. Set Up Database

#### Create a Supabase Project

1. Go to [supabase.com](https://supabase.com/) and create account
2. Create a new project
3. Note your project URL and API keys from **Settings → API**

#### Apply Database Schema

```bash
# Option 1: Using psql (if direct database access is available)
psql "$DATABASE_URL" < schema.sql

# Option 2: Using Supabase SQL Editor
# Copy the contents of schema.sql and paste into the SQL Editor
# Click "Run" to execute
```

Verify tables were created:

```sql
-- Run this in Supabase SQL Editor
SELECT table_name FROM information_schema.tables 
WHERE table_schema = 'public';
```

You should see: `users`, `tunnels`, `api_keys`, `tunnel_requests`, `analytics_daily`, `revoked_tokens`

### 4. Configure Environment Variables

Create `.env` file in the `libneedle` directory:

```bash
cp .env.example .env
```

Edit `.env` with your values:

```bash
# Database (REQUIRED)
SUPABASE_URL=https://xxxxx.supabase.co
SUPABASE_ANON_KEY=eyJhbG...  # From Supabase Settings → API
SUPABASE_SERVICE_ROLE_KEY=eyJhbG...  # Keep this secret!

# Security (REQUIRED)
# Generate with: openssl rand -hex 32
JWT_SECRET=your-256-bit-random-secret

# Server Addressing (Optional - defaults shown)
API_ADDR=0.0.0.0:3000
SSH_ADDR=0.0.0.0:2222
DOMAIN=yourdomain.com  # Or localhost for development

# Tunnel Limits (Optional)
MAX_TUNNELS_PER_IP=5
GLOBAL_TUNNEL_LIMIT=1000

# HTTP Timeouts (Optional)
HTTP_READ_TIMEOUT_SECS=10
HTTP_WRITE_TIMEOUT_SECS=10

# Tier Limits (Optional)
FREE_TIER_LIMIT=3
PRO_TIER_LIMIT=50
ENTERPRISE_TIER_LIMIT=500

# SSH Security (Optional)
MIN_SSH_PORT=1024

# Logging (Optional)
RUST_LOG=needle=info,tower_http=info
```

> [!WARNING]
> **Keep `SUPABASE_SERVICE_ROLE_KEY` and `JWT_SECRET` secret!**
> 
> These keys bypass Row Level Security and can access all data. Never commit them to git or expose them publicly.

### 5. Build the Backend

Development build (with debug symbols):

```bash
cargo build
```

Production build (optimized):

```bash
cargo build --release
```

The binary will be at:
- Debug: `target/debug/needle-server`
- Release: `target/release/needle-server`

### 6. Run Tests

Verify everything is working:

```bash
cargo test --workspace
```

All tests should pass.

### 7. Start the Server

Development mode:

```bash
cargo run --bin needle-server
```

Production mode (using release binary):

```bash
./target/release/needle-server
```

You should see:

```
INFO needle_server: loaded configuration
  api=0.0.0.0:3000 ssh=0.0.0.0:2222 domain=yourdomain.com
INFO needle_server: starting api server addr=0.0.0.0:3000
INFO needle_server: starting ssh server addr=0.0.0.0:2222
```

## Frontend Installation

### 1. Install Node.js

Download from [nodejs.org](https://nodejs.org/) or use a package manager:

```bash
# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# macOS
brew install node@20

# Verify
node --version  # Should be 20+
npm --version
```

### 2. Install Dependencies

```bash
cd needleui
npm install
```

### 3. Configure Frontend

Create `needleui/.env`:

```bash
VITE_API_URL=http://localhost:3000
```

For production, set this to your actual API URL:

```bash
VITE_API_URL=https://api.yourdomain.com
```

### 4. Start Development Server

```bash
npm run dev
```

Open [http://localhost:5173](http://localhost:5173)

### 5. Build for Production

```bash
npm run build
```

Static files will be in `dist/` directory. Serve with nginx, Apache, or any static file server.

## Verification

### 1. Check Health Endpoint

```bash
curl http://localhost:3000/health
```

Expected response:

```json
{
  "status": "ok",
  "timestamp": "2026-02-10T16:30:00Z"
}
```

### 2. Check Metrics Endpoint

```bash
curl http://localhost:3000/metrics
```

Should return Prometheus metrics format.

### 3. Create a Test User

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "username": "testuser",
    "password": "SecurePass123!"
  }'
```

Expected response:

```json
{
  "success": true,
  "user": {
    "id": "uuid-here",
    "email": "test@example.com",
    "username": "testuser"
  }
}
```

## Troubleshooting

### Build Errors

**"linker 'cc' not found"**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS (install Xcode Command Line Tools)
xcode-select --install
```

**"failed to fetch" during cargo build**
```bash
# Update cargo index
cargo update
rm -rf ~/.cargo/registry/index/*
cargo build
```

### Runtime Errors

**"SUPABASE_URL is required but not set"**
- Verify `.env` file exists in `libneedle/` directory
- Check variable names match exactly (case-sensitive)
- Try running: `source .env && cargo run`

**"Invalid configuration: invalid API address"**
- Verify `API_ADDR` format is `host:port` (e.g., `0.0.0.0:3000`)
- Check port is not already in use: `lsof -i :3000`

**"Database connection failed"**
- Verify Supabase URL is correct
- Check API keys haven't expired
- Ensure schema was applied successfully
- Test connection: `curl -H "apikey: $SUPABASE_ANON_KEY" "$SUPABASE_URL/rest/v1/users?limit=1"`

### Frontend Issues

**"Cannot connect to API"**
- Verify backend is running
- Check `VITE_API_URL` in frontend `.env`
- Check CORS configuration if backend and frontend are on different domains

**"npm install" fails**
- Clear cache: `npm cache clean --force`
- Delete `node_modules` and `package-lock.json`, then run `npm install` again
- Try: `npm install --legacy-peer-deps`

## Next Steps

- [Creating Tunnels](./creating-tunnels.md) - Learn how to create and manage tunnels
- [Configuration](./configuration.md) - Detailed configuration reference
- [Dashboard](./dashboard.md) - Using the web interface
