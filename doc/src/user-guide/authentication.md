# Authentication

Needle supports multiple authentication methods for both the web dashboard and SSH tunnels.

## Authentication Methods

| Method | Use Case | Supported |
|--------|----------|-----------|
| Email/Password | Dashboard login | ✅ Yes |
| OAuth (GitHub) | Dashboard login | 🚧 Planned |
| OAuth (Google) | Dashboard login | 🚧 Planned |
| API Keys | SSH tunnel auth | ✅ Yes |
| JWT Tokens | API requests | ✅ Yes |

## Email/Password Authentication

### Registration

Create a new account via API:

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "you@example.com",
    "username": "yourusername",
    "password": "SecurePassword123!"
  }'
```

Response:

```json
{
  "success": true,
  "user": {
    "id": "uuid-here",
    "email": "you@example.com",
    "username": "yourusername",
    "tier": "free"
  }
}
```

### Login

Authenticate and receive a JWT token:

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "you@example.com",
    "password": "SecurePassword123!"
  }'
```

Response:

```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "uuid-here",
    "email": "you@example.com",
    "username": "yourusername"
  }
}
```

### Password Requirements

- **Minimum length**: 8 characters
- **Complexity**: Mix of uppercase, lowercase, numbers recommended
- **Not allowed**: Common passwords (password123, etc.)

## JWT Tokens

### Token Structure

JWT tokens contain:
- **User ID** - Unique user identifier
- **Email** - User's email address
- **Tier** - Free, Pro, or Enterprise
- **Issued At** - Token creation timestamp
- **Expires** - Expiration timestamp (default: 24 hours)

### Using JWT Tokens

Include the token in the `Authorization` header:

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     http://localhost:3000/api/tunnels
```

### Token Expiration

- Default: **24 hours**
- After expiration, you must log in again
- Refresh tokens are not currently supported

### Token Revocation

> [!WARNING]
> JWT tokens cannot be revoked before expiration!

This is a known limitation. Workarounds:
- Keep token lifetime short (24 hours)
- Change `JWT_SECRET` to invalidate all tokens (breaks all sessions)
- Use Redis blacklist (future enhancement)

## API Keys

API keys are long-lived credentials for SSH tunnel authentication.

### Creating API Keys

**Via Dashboard:**
1. Log in to the dashboard
2. Go to **Settings → API Keys**
3. Click **Create New Key**
4. Enter a name and optional expiration
5. Copy the key (only shown once!)

**Via API:**

```bash
curl -X POST http://localhost:3000/api/keys \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Laptop",
    "expires_at": "2027-01-01T00:00:00Z"
  }'
```

Response:

```json
{
  "key": "needle_a1b2c3d4e5f6g7h8i9j0",
  "id": "uuid-here",
  "name": "My Laptop",
  "created_at": "2026-02-10T12:00:00Z"
}
```

> [!IMPORTANT]
> **Save the key immediately!**
> 
> The full key is only shown once. Only the prefix (e.g., `needle_a1b2...`) is stored.

### Using API Keys

Use API keys as the SSH username:

```bash
ssh -R 80:localhost:3000 \
    tunnel@yourdomain.com -p 2222 \
    -o "User=needle_a1b2c3d4e5f6g7h8i9j0"
```

### Key Format

- **Prefix**: `needle_` (identifies as Needle API key)
- **Length**: 32 characters (plus prefix)
- **Characters**: Alphanumeric (no special characters)

Example: `needle_a1b2c3d4e5f6g7h8i9j0k1l2m3n4`

### Key Scopes

API keys have the following permissions:
- **Create tunnels** (`tunnels:write`)
- **List own tunnels** (`tunnels:read`)
- **Delete own tunnels** (`tunnels:write`)

API keys **cannot**:
- Access other users' tunnels
- Create new API keys
- Modify account settings
- Access analytics API

### Key Rotation

Best practice: **Rotate keys every 90 days**

1. Create a new API key
2. Update your SSH config to use the new key
3. Test that tunnels work with the new key
4. Revoke the old key

### Revoking Keys

**Via Dashboard:**
1. Go to **Settings → API Keys**
2. Find the key to revoke
3. Click **Revoke**
4. Confirm

**Via API:**

```bash
curl -X DELETE http://localhost:3000/api/keys/KEY_ID \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

All active tunnels using that key will immediately disconnect.

## OAuth (Planned)

> [!NOTE]
> OAuth support is planned but not yet implemented.

Planned OAuth providers:
- **GitHub** - Authenticate with your GitHub account
- **Google** - Authenticate with your Google account

This will allow dashboard login without creating a password.

## Security Best Practices

### Passwords

- ✅ **Use a password manager** to generate strong passwords
- ✅ **Never reuse passwords** across services
- ✅ **Enable 2FA** (when available - planned feature)
- ❌ **Don't share passwords** with teammates

### JWT Tokens

- ✅ **Store securely** in browser localStorage or secure cookie
- ✅ **Never commit to git** or expose in URLs
- ✅ **Refresh when expired** by logging in again
- ❌ **Don't share tokens** between users

### API Keys

- ✅ **One key per device/environment** (laptop, CI, staging)
- ✅ **Set expiration dates** for temporary access
- ✅ **Revoke immediately** if compromised
- ✅ **Rotate regularly** (every 90 days)
- ❌ **Never commit to git** or include in client-side code

### HTTPS Only

> [!CAUTION]
> **Always use HTTPS in production!**

- Credentials sent over HTTP can be intercepted
- Use a reverse proxy (nginx) with SSL/TLS
- Let's Encrypt provides free certificates

## SSH Authentication

When creating tunnels, Needle uses Public Key Authentication:

### How It Works

1. You send your API key in the SSH username field
2. Needle validates the API key against the database
3. If valid, the SSH connection is accepted
4. Further authentication is not required for the session

### No SSH Keys Required

Unlike traditional SSH, you **don't need SSH key pairs**. The API key serves as your authentication credential.

## Rate Limiting

Authentication endpoints are rate limited to prevent brute force attacks:

- **Login**: 5 attempts per IP per minute
- **Register**: 3 accounts per IP per hour
- **API Key Creation**: 10 keys per user per hour

If you hit rate limits, wait a few minutes and try again.

## Account Recovery

> [!WARNING]
> **Password reset is not yet implemented.**

If you forget your password:
1. Contact the server administrator
2. They can manually reset your password in the database
3. Check for future password reset functionality

## Next Steps

- [Creating Tunnels](./creating-tunnels.md) - Use your API key to create tunnels
- [Dashboard](./dashboard.md) - Manage API keys in the web UI
- [Security](../operations/security.md) - Advanced security practices
