# Needle Deployment Guide

## Quick Start

### Prerequisites
- PostgreSQL database (via Supabase)
- Rust 1.70+ and Cargo
- Linux server with public IP
- Domain name with DNS access

### Environment Variables

Create a `.env` file with the following required variables:

```bash
# Database (Required)
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Security (Required)
JWT_SECRET=your-random-256-bit-secret  # Generate with: openssl rand -hex 32

# Server Addressing (Optional - defaults shown)
API_ADDR=0.0.0.0:3000
SSH_ADDR=0.0.0.0:2222
DOMAIN=yourdomain.com

# Tunnel Limits (Optional - defaults shown)
MAX_TUNNELS_PER_IP=5
GLOBAL_TUNNEL_LIMIT=1000

# HTTP Proxy Timeouts (Optional - defaults shown)
HTTP_READ_TIMEOUT_SECS=10
HTTP_WRITE_TIMEOUT_SECS=10

# Tier Enforcement (Optional - defaults shown)
FREE_TIER_LIMIT=3
PRO_TIER_LIMIT=50
ENTERPRISE_TIER_LIMIT=500

# SSH Security (Optional - default shown)
MIN_SSH_PORT=1024

# Logging (Optional)
RUST_LOG=needle=info,tower_http=info

# CORS (Optional - for frontend)
CORS_ORIGIN=https://yourdomain.com
```

### Database Setup

1. Apply schema to your Supabase project:
```bash
psql $DATABASE_URL < schema.sql
```

2. Verify tables created:
```sql
\dt  -- Should show: users, tunnels, api_keys, tunnel_requests, analytics_daily, revoked_tokens
```

### Building

```bash
# Development build
cargo build

# Production build (optimized)
cargo build --release

# Run tests
cargo test --workspace
```

### Running

```bash
# Development
cargo run --bin needle-server

# Production (from release binary)
./target/release/needle-server
```

The server will:
1. Validate configuration on startup
2. Bind API server on `API_ADDR`
3. Bind SSH server on `SSH_ADDR`
4. Log configuration summary
5. Expose `/metrics` endpoint for monitoring

### Health Checks

```bash
# Basic health check
curl http://localhost:3000/health

# Metrics (Prometheus format)
curl http://localhost:3000/metrics
```

## Production Deployment

###SystemD Service

Create `/etc/systemd/system/needle.service`:

```ini
[Unit]
Description=Needle Tunneling Service
After=network.target

[Service]
Type=simple
User=needle
Group=needle
WorkingDirectory=/opt/needle
Environment="RUST_LOG=needle=info"
EnvironmentFile=/opt/needle/.env
ExecStart=/opt/needle/bin/needle-server
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/needle

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable needle
sudo systemctl start needle
sudo systemctl status needle
```

### Reverse Proxy (Nginx)

```nginx
# API server
server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    location /metrics {
        deny all;  # Restrict metrics to internal monitoring only
    }
}

# Wildcard for tunnels
server {
    listen 443 ssl http2;
    server_name *.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### Monitoring Setup (Prometheus)

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'needle'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

Key metrics to alert on:
- `needle_tunnels_active` - Active tunnel count
- `needle_auth_failures_total` - Authentication failures
- `needle_errors_total` - Error rate
- `needle_http_request_duration_seconds` - Request latency

### Firewall Configuration

```bash
# Allow API (HTTPS via reverse proxy)
sudo ufw allow 443/tcp

# Allow SSH tunneling
sudo ufw allow 2222/tcp

# Block direct API port (use reverse proxy only)
sudo ufw deny 3000/tcp
```

## Operational Procedures

### Upgrading

```bash
# 1. Build new version
cd /opt/needle
git pull
cargo build --release

# 2. Stop service
sudo systemctl stop needle

# 3. Backup database (optional but recommended)
pg_dump $DATABASE_URL > backup-$(date +%Y%m%d).sql

# 4. Run migrations if any (check CHANGELOG)
psql $DATABASE_URL < migrations/YYYYMMDD_description.sql

# 5. Start service
sudo systemctl start needle

# 6. Verify health
curl http://localhost:3000/health
```

### Rolling Back

```bash
# 1. Stop service
sudo systemctl stop needle

# 2. Restore previous binary
cp bin/needle-server.backup bin/needle-server

# 3. Restore database if schema changed
psql $DATABASE_URL < backup-YYYYMMDD.sql

# 4. Start service
sudo systemctl start needle
```

### Log Management

```bash
# View logs
sudo journalctl -u needle -f

# View last 100 lines
sudo journalctl -u needle -n 100

# View logs for specific time range
sudo journalctl -u needle --since "2024-01-01" --until "2024-01-02"

# Filter by log level
sudo journalctl -u needle -p err  # Errors only
```

### Database Maintenance

```bash
# Clean up expired revoked tokens (run daily via cron)
psql $DATABASE_URL << EOF
DELETE FROM revoked_tokens WHERE expires_at < NOW();
EOF

# Analyze tunnel usage
psql $DATABASE_URL << EOF
SELECT protocol, COUNT(*) as count, AVG(target_port) as avg_port
FROM tunnels
WHERE created_at > NOW() - INTERVAL '7 days'
GROUP BY protocol;
EOF
```

## Troubleshooting

### Server Won't Start

**Check configuration:**
```bash
# Validate env vars are set
env | grep -E '(SUPABASE|JWT_SECRET)'

# Check config validation errors in logs
sudo journalctl -u needle -n 50 | grep -i "invalid configuration"
```

**Common issues:**
- Invalid `DOMAIN` format
- Missing required env vars
- Port already in use
- Database connection failure

### High Auth Failures

```bash
# Check auth failure metrics
curl http://localhost:3000/metrics | grep auth_failures

# View IP addresses with failures
sudo journalctl -u needle | grep "authentication failed" | awk '{print $NF}' | sort | uniq -c
```

Consider adding IP-based rate limiting if seeing brute force attacks.

### Database Connection Issues

```bash
# Test Supabase connection
curl -H "apikey: $SUPABASE_ANON_KEY" \
     -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
     "$SUPABASE_URL/rest/v1/users?limit=1"
```

Check Supabase dashboard for:
- Row-level security policies enabled
- Service role key permissions
- Connection limits

### Tunnel Creation Failures

```bash
# Check error metrics
curl http://localhost:3000/metrics | grep needle_errors_total

# Common causes:
# - Tier limit exceeded
# - Global capacity reached
# - Database write failures
# - Port binding conflicts
```

## Security Checklist

- [ ] JWT_SECRET is cryptographically random (32+ bytes)
- [ ] SUPABASE_SERVICE_ROLE_KEY not exposed in logs
- [ ] SSL certificates valid and auto-renewing
- [ ] Firewall rules restrict direct API access
- [ ] `/metrics` endpoint access controlled
- [ ] Database RLS policies enabled
- [ ] Regular security updates applied
- [ ] Monitoring alerts configured

## Performance Tuning

### Database Indexes

Verify these indexes exist for optimal performance:
```sql
-- Check existing indexes
\di

-- Should include:
-- idx_tunnels_subdomain
-- idx_tunnels_user
-- idx_api_keys_hash
-- idx_revoked_tokens_expires
-- idx_analytics_daily_tunnel
```

### Connection Limits

Adjust based on load:
```bash
GLOBAL_TUNNEL_LIMIT=5000  # Increase for high-traffic deployments
MAX_TUNNELS_PER_IP=10     # Increase for trusted users
```

### HTTP Timeouts

For slow upstream services:
```bash
HTTP_READ_TIMEOUT_SECS=30   # Increase for slow backends
HTTP_WRITE_TIMEOUT_SECS=30
```

## Support

- Documentation: https://github.com/yourorg/needle/docs
- Issues: https://github.com/yourorg/needle/issues
- Metrics: http://localhost:3000/metrics
- Health: http://localhost:3000/health
