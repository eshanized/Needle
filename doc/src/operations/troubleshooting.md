# Troubleshooting

Common issues and solutions for Needle.

[_See complete troubleshooting guide in DEPLOY.md_]

## Server Won't Start

### "SUPABASE_URL is required but not set"

**Cause**: Missing required environment variable

**Solution**:
```bash
# Check .env file exists
ls -la /opt/needle/.env

# Verify variable is set
env | grep SUPABASE_URL

# Load .env and restart
source .env && cargo run
```

### "Invalid configuration: invalid API address"

**Cause**: Malformed `API_ADDR` or `SSH_ADDR`

**Solution**:
```bash
# Correct format: host:port
API_ADDR=0.0.0.0:3000
SSH_ADDR=0.0.0.0:2222

# Not valid
API_ADDR=3000  # Missing host
SSH_ADDR=0.0.0.0  # Missing port
```

### "Port already in use"

**Cause**: Another process is using port 3000 or 2222

**Solution**:
```bash
# Find process using port
sudo lsof -i :3000
sudo lsof -i :2222

# Kill process
sudo kill <PID>

# Or change ports in .env
API_ADDR=0.0.0.0:3001
SSH_ADDR=0.0.0.0:2223
```

## Authentication Issues

### "Authentication failed" when creating tunnel

**Causes**:
1. Invalid API key
2. Expired API key
3. Wrong format

**Solutions**:
```bash
# 1. Verify API key is correct
# - Check dashboard → API Keys
# - Copy exact key including prefix

# 2. Check key hasn't expired
# - View "Expires At" in dashboard
# - Create new key if expired

# 3. Verify format
# Correct: needle_a1b2c3d4...
# Wrong: a1b2c3d4... (missing prefix)
```

### "Invalid credentials" on login

**Causes**:
1. Wrong email/password
2. Account doesn't exist
3. Password changed recently

**Solutions**:
```bash
# Verify email is correct
curl -X POST http://localhost:3000/api/auth/login \
  -d '{"email":"EXACT_EMAIL","password":"..."}'

# If forgotten, contact admin (no self-service reset yet)
```

## Tunnel Issues

### Tunnel created but can't access URL

**Check**: DNS is configured

```bash
# Verify wildcard DNS works
nslookup abc123.yourdomain.com
# Should resolve to your server IP
```

**Check**: Local app is running

```bash
# Verify app is listening
curl http://localhost:3000
```

**Check**: Firewall allows traffic

```bash
# Ensure port 3000 is open internally
sudo ufw status
```

### SSL certificate errors

**Cause**: Let's Encrypt certificate not issued for wildcard

**Solution**:
```bash
# Get wildcard certificate
sudo certbot certonly --manual \
  --preferred-challenges dns \
  -d '*.yourdomain.com' \
  -d 'yourdomain.com'

# Follow prompts to add DNS TXT record
```

### "Rate limit exceeded"

**Cause**: Too many tunnel creation attempts

**Wait**: Rate limits reset after configured time period

**Check tier limits**:
- Free: 3 concurrent tunnels
- Pro: 50 concurrent tunnels

## Database Issues

### "Database connection failed"

**Check Supabase URL**:
```bash
curl -H "apikey: $SUPABASE_ANON_KEY" \
  "$SUPABASE_URL/rest/v1/users?limit=1"
```

**Check API keys**:
- Verify anon key and service role key are correct
- Check keys haven't been rotated in Supabase dashboard

**Check network**:
- Verify server can reach Supabase (no firewall blocking)
- Check Supabase project is active (not paused)

### "Schema not found" errors

**Cause**: Database schema not applied

**Solution**:
```bash
# Apply schema
psql "$DATABASE_URL" < libneedle/schema.sql

# Verify tables exist
psql "$DATABASE_URL" -c "\\dt"
```

## Performance Issues

### High latency

**Check Prometheus metrics**:
```bash
curl http://localhost:3000/metrics | grep latency
```

**Common causes**:
- Slow backend (check local app performance)
- Network congestion
- Database slow queries

**Solutions**:
- Increase timeouts if backends are legitimately slow
- Add database indexes
- Upgrade server resources

### Memory usage growing

**Check logs for leaks using journalctl`:
```bash
journalctl -u needle | grep -i "memory\|oom"
```

**Solutions**:
- Restart service regularly (cron)
- Increase server RAM
- Report as bug if leak confirmed

## Dashboard Issues

###  "Cannot connect to backend"

**Check backend is running**:
```bash
curl http://localhost:3000/health
```

**Check CORS**:
```bash
# In backend .env
CORS_ORIGIN=https://dashboard.yourdomain.com
```

**Check frontend config**:
```bash
# In needleui/.env
VITE_API_URL=http://localhost:3000
```

### Data not updating

**Hard refresh**:
- Chrome/Firefox: `Ctrl+Shift+R`
- Mac: `Cmd+Shift+R`

**Check WebSocket connection**:
- Open browser DevTools → Network tab
- Look for WebSocket connection
- Should show "101 Switching Protocols"

## Logs and Debugging

### View logs

```bash
# Real-time
sudo journalctl -u needle -f

# Last 100 lines
sudo journalctl -u needle -n 100

# Errors only
sudo journalctl -u needle -p err

# Specific time range
sudo journalctl -u needle --since "1 hour ago"
```

### Increase log verbosity

```bash
# In .env
RUST_LOG=needle=debug,tower_http=debug

# Restart service
sudo systemctl restart needle
```

### Enable request logging

Requests are automatically logged. Check metrics:

```bash
curl http://localhost:3000/metrics | grep needle_http_requests_total
```

## Getting Help

Still stuck? Get help:

1. **Check logs first** - Most issues show clear errors
2. **Search GitHub issues** - Someone may have encountered this
3. **Open new issue** - Include logs and configuration (redact secrets!)
4. **Community forum** - Ask in discussions

When reporting issues, include:
- Needle version (`git rev-parse HEAD`)
- OS and version
- Relevant logs (without secrets!)
- Steps to reproduce

## Next Steps

- [Monitoring](./monitoring.md) - Proactive issue detection
- [Security](./security.md) - Security-related troubleshooting
- [Deployment](./deployment.md) - Deployment issues
