# Security

Production security best practices for Needle.

## Authentication Security

### JWT Secrets

**Generate strong secrets:**

```bash
# Generate 256-bit random secret
openssl rand -hex 32
```

**Rotate secrets:**

- Rotate `JWT_SECRET` every 90 days
- Rotation invalidates all existing tokens
- Plan rotation during low-traffic periods

### API Keys

**Storage:**
- Full key shown only once at creation
- Only SHA256 hash stored in database
- Display only prefix (e.g., `needle_abc123...`)

**Rotation:**
- Rotate keys every 90 days
- One key per device/environment
- Revoke immediately if compromised

### Password Policy

Current (basic):
- Minimum 8 characters
- Argon2 hashing

Recommendations:
- Require complex passwords (uppercase, lowercase, numbers, symbols)
- Implement password strength meter
- Add rate limiting on password attempts (currently implemented)

## Network Security

### HTTPS Only

**Never run without HTTPS in production!**

- Use Let's Encrypt for free SSL certificates
- Configure nginx as reverse proxy
- Redirect HTTP → HTTPS

### Firewall

```bash
# Allow only necessary ports
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 2222/tcp  # SSH Tunnels
sudo ufw enable
```

### SSH Port Security

**Restrict forwarding ports:**

```bash
# In .env
MIN_SSH_PORT=1024
```

This prevents users from forwarding privileged ports (< 1024).

## Database Security

### Row-Level Security

Supabase RLS policies ensure users can only access their own data:

```sql
CREATE POLICY "tunnels_owner_access" ON tunnels
    FOR ALL USING (user_id = auth.uid());
```

**Verify RLS is enabled:**

```sql
SELECT tablename, rowsecurity 
FROM pg_tables 
WHERE schemaname = 'public';
-- rowsecurity should be 't' (true) for all tables
```

### Service Role Key

> [!CAUTION]
> **Never expose SUPABASE_SERVICE_ROLE_KEY!**

This key bypasses RLS. Protect it like root passwords:

- ✅ Store in `.env` (not in code)
- ✅ Restrict file permissions (`chmod 600 .env`)
- ✅ Use different keys for dev/staging/prod
- ❌ Never commit to git
- ❌ Never log or expose in error messages

### Connection Security

- Use SSL/TLS for database connections
- Limit connections to trusted IPs (Supabase firewall rules)
-Regularly review database access logs

## Application Security

### Input Validation

All user input is validated:

- **Subdomains** - Regex validation, length checks
- **Emails** - Format validation
- **Ports** - Range validation (> MIN_SSH_PORT)

### Rate Limiting

Protection against abuse:

- **Login** - 5 attempts/minute per IP
- **Register** - 3 accounts/hour per IP
- **Tunnel creation** - Configurable per-IP limit

### CORS

Restrict origins in production:

```bash
# .env
CORS_ORIGIN=https://dashboard.yourdomain.com
```

Don't use `*` in production!

## Secrets Management

### Environment Variables

**Secure storage:**

```bash
# Set restrictive permissions
chmod 600 /opt/needle/.env
chown needle:needle /opt/needle/.env
```

**Never log secrets:**

Audit code for logging statements that might expose:
- `JWT_SECRET`
- `SUPABASE_SERVICE_ROLE_KEY`
- `password_hash`
- API keys

### Secrets in Docker

Use Docker secrets instead of environment variables:

```yaml
# docker-compose.yml
version: '3.8'
services:
  needle:
    image: needle:latest
    secrets:
      - jwt_secret
      - supabase_key

secrets:
  jwt_secret:
    file: ./secrets/jwt_secret.txt
  supabase_key:
    file: ./secrets/supabase_key.txt
```

## Vulnerability Management

### Dependency Scanning

Install cargo-audit:

```bash
cargo install cargo-audit
```

Run regularly:

```bash
cargo audit
```

Fix vulnerabilities:

```bash
cargo update -p vulnerable_crate
```

### Security Updates

- Subscribe to Rust security advisories
- Monitor dependency CVEs
- Update dependencies monthly
- Apply critical security patches immediately

## Incident Response

### Compromised API Key

1. User reports compromised key
2. Revoke key immediately (dashboard or API)
3. All tunnels using that key disconnect
4. User creates new key
5. Monitor for suspicious activity

### Compromised JWT_SECRET

1. Generate new JWT_SECRET
2. Update `.env` file
3. Restart service
4. **All users logged out** (all tokens invalidated)
5. Notify users via email

### Data Breach

1. **Containment** - Disconnect affected systems
2. **Assessment** - Determine scope and impact
3. **Notification** - Notify affected users (GDPR compliance)
4. **Remediation** - Patch vulnerability
5. **Review** - Post-mortem and process improvements

## Compliance

### GDPR

For EU users:

- **Right to access** - Provide user data export
- **Right to deletion** - Delete user and all associated data
- **Data retention** - Implement retention policies
- **Privacy policy** - Clearly state data usage

### Logging

**Do log**:
- Authentication attempts (success/failure)
- Tunnel creation/deletion
- API requests (method, path, status)

**Don't log**:
- Passwords or password hashes
- API keys or JWT tokens
- Tunnel traffic content
- Personal data unnecessarily

## Security Checklist

Before production:

- [ ] All secrets are cryptographically random
- [ ] HTTPS configured with valid certificates
- [ ] Firewall rules restrict unnecessary ports
- [ ] `/metrics` endpoint restricted to monitoring system
- [ ] Database RLS policies enabled and tested
- [ ] Environment-specific credentials (no shared secrets)
- [ ] Dependency audit passes (`cargo audit`)
- [ ] Rate limiting configured
- [ ] CORS restricted to dashboard domain
- [ ] Logging audited for sensitive data
- [ ] Incident response plan documented
- [ ] Regular security updates scheduled

## Next Steps

- [Production Certification](./production-certification.md) - Full audit checklist
- [Monitoring](./monitoring.md) - Detect security incidents
- [Troubleshooting](./troubleshooting.md) - Security-related issues
