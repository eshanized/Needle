# API Reference

Complete REST API documentation for Needle tunneling service.

## Base URL

```
http://localhost:3000/api
```

In production, replace with your actual domain (e.g., `https://api.yourdomain.com/api`).

## Authentication

Most endpoints require a JWT token in the `Authorization` header:

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Get a token by calling `POST /api/auth/login`.

## Endpoints

### Authentication

#### POST /api/auth/register

Create a new user account.

**Request:**
```json
{
  "email": "user@example.com",
  "username": "johndoe",
  "password": "SecurePassword123!"
}
```

**Response:** `201 Created`
```json
{
  "success": true,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "username": "johndoe",
    "tier": "free",
    "created_at": "2026-02-10T12:00:00Z"
  }
}
```

**Errors:**
- `400` - Invalid email format or password too weak
- `409` - Email or username already exists

---

#### POST /api/auth/login

Authenticate andreceive JWT token.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Response:** `200 OK`
```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "username": "johndoe",
    "tier": "free"
  }
}
```

**Errors:**
- `401` - Invalid credentials
- `429` - Rate limit exceeded (5 attempts/minute)

---

### Tunnels

#### GET /api/tunnels

List all tunnels for the authenticated user.

**Headers:**
```http
Authorization: Bearer {token}
```

**Response:** `200 OK`
```json
{
  "tunnels": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440000",
      "subdomain": "abc123",
      "custom_domain": null,
      "target_port": 3000,
      "protocol": "http",
      "is_active": true,
      "is_persistent": false,
      "created_at": "2026-02-10T12:00:00Z",
      "last_active": "2026-02-10T14:30:00Z"
    }
  ]
}
```

---

#### POST /api/tunnels

Create a new tunnel (alternative to SSH command).

**Headers:**
```http
Authorization: Bearer {token}
Content-Type: application/json
```

**Request:**
```json
{
  "subdomain": "myapp",
  "target_port": 3000,
  "protocol": "http",
  "is_persistent": false
}
```

All fields are optional:
- `subdomain` - Custom subdomain (requires Pro tier). Omit for random.
- `target_port` - Default: 80
- `protocol` - Default: "http"
- `is_persistent` - Default: false

**Response:** `201 Created`
```json
{
  "subdomain": "myapp",
  "url": "https://myapp.yourdomain.com",
  "bind_addr": "127.0.0.1:8081"
}
```

**Errors:**
- `400` - Invalid subdomain format
- `403` - Tier limit exceeded
- `409` - Subdomain already taken

---

#### DELETE /api/tunnels/:subdomain

Delete a tunnel by subdomain.

**Headers:**
```http
Authorization: Bearer {token}
```

**Response:** `204 No Content`

**Errors:**
- `403` - Tunnel belongs to another user
- `404` - Tunnel not found

---

### API Keys

#### GET /api/keys

List all API keys for the authenticated user.

**Headers:**
```http
Authorization: Bearer {token}
```

**Response:** `200 OK`
```json
{
  "keys": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "name": "My Laptop",
      "key_prefix": "needle_a1b2c3d4",
      "scopes": ["tunnels:read", "tunnels:write"],
      "last_used": "2026-02-10T14:00:00Z",
      "expires_at": null,
      "created_at": "2026-02-01T10:00:00Z"
    }
  ]
}
```

---

#### POST /api/keys

Generate a new API key.

**Headers:**
```http
Authorization: Bearer {token}
Content-Type: application/json
```

**Request:**
```json
{
  "name": "CI/CD Pipeline",
  "expires_at": "2027-01-01T00:00:00Z"
}
```

Fields:
- `name` - Required. Descriptive label
- `expires_at` - Optional. ISO 8601 timestamp

**Response:** `201 Created`
```json
{
  "key": "needle_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6",
  "id": "770e8400-e29b-41d4-a716-446655440000",
  "name": "CI/CD Pipeline",
  "created_at": "2026-02-10T15:00:00Z"
}
```

> **IMPORTANT:** Save the `key` value immediately! It's only shown once.

---

#### DELETE /api/keys/:id

Revoke an API key.

**Headers:**
```http
Authorization: Bearer {token}
```

**Response:** `204 No Content`

All active tunnels using this key will disconnect immediately.

---

### Analytics

#### GET /api/analytics/daily

Get daily aggregated statistics for a tunnel.

**Headers:**
```http
Authorization: Bearer {token}
```

**Query Parameters:**
- `tunnel_id` - Required. UUID of the tunnel
- `from` - Optional. Start date (YYYY-MM-DD)
- `to` - Optional. End date (YYYY-MM-DD)

**Example:**
```
GET /api/analytics/daily?tunnel_id=660e8400-e29b-41d4-a716-446655440000&from=2026-02-01&to=2026-02-10
```

**Response:** `200 OK`
```json
{
  "analytics": [
    {
      "date": "2026-02-10",
      "total_requests": 1234,
      "total_bytes_in": 5678900,
      "total_bytes_out": 9876543,
      "avg_latency_ms": 45,
      "error_count": 12,
      "unique_ips": 67
    }
  ]
}
```

---

### Traffic Inspector

#### GET /api/inspector/requests

Get HTTP request logs for a tunnel.

**Headers:**
```http
Authorization: Bearer {token}
```

**Query Parameters:**
- `tunnel_id` - Required. UUID of the tunnel
- `limit` - Optional. Max results (default: 100, max: 1000)
- `offset` - Optional. Pagination offset
- `method` - Optional. Filter by HTTP method
- `status` - Optional. Filter by status code

**Example:**
```
GET /api/inspector/requests?tunnel_id=660e&limit=50&method=POST
```

**Response:** `200 OK`
```json
{
  "requests": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440000",
      "method": "POST",
      "path": "/api/users",
      "status_code": 201,
      "latency_ms": 34,
      "request_size": 256,
      "response_size": 128,
      "client_ip": "203.0.113.42",
      "timestamp": "2026-02-10T15:30:00Z"
    }
  ],
  "total": 1234,
  "limit": 50,
  "offset": 0
}
```

---

#### GET /api/inspector/requests/:id

Get full details for a specific request.

**Headers:**
```http
Authorization: Bearer {token}
```

**Response:** `200 OK`
```json
{
  "id": "880e8400-e29b-41d4-a716-446655440000",
  "method": "POST",
  "path": "/api/users",
  "status_code": 201,
  "latency_ms": 34,
  "request_headers": {
    "Content-Type": "application/json",
    "Authorization": "Bearer ..."
  },
  "response_headers": {
    "Content-Type": "application/json",
    "X-Request-ID": "abc123"
  },
  "request_body": "{\"username\":\"johndoe\"}",
  "response_body": "{\"id\":\"123\",\"username\":\"johndoe\"}",
  "client_ip": "203.0.113.42",
  "timestamp": "2026-02-10T15:30:00Z"
}
```

---

### Health & Metrics

#### GET /health

Check server health (no auth required).

**Response:** `200 OK`
```json
{
  "status": "ok",
  "timestamp": "2026-02-10T16:00:00Z"
}
```

---

#### GET /metrics

Prometheus metrics export (no auth required).

**Response:** `200 OK` (Prometheus text format)
```
# HELP needle_tunnels_active Number of active tunnels
# TYPE needle_tunnels_active gauge
needle_tunnels_active 42

# HELP needle_http_requests_total Total HTTP requests
# TYPE needle_http_requests_total counter
needle_http_requests_total{method="GET",status="200"} 1234
```

## Error Format

All errors follow this format:

```json
{
  "error": "descriptive error message"
}
```

## Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful deletion) |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (missing/invalid token) |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 409 | Conflict (subdomain already taken) |
| 429 | Too Many Requests (rate limited) |
| 500 | Internal Server Error |

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| `POST /api/auth/login` | 5/minute per IP |
| `POST /api/auth/register` | 3/hour per IP |
| `POST /api/keys` | 10/hour per user |
| All other endpoints | 100/minute per user |

## Next Steps

- [Testing](./testing.md) - Testing the API
- [Contributing](./contributing.md) - Adding new endpoints
