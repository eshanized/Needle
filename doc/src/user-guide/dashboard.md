# Web Dashboard

The Needle dashboard provides a modern web interface for managing tunnels, viewing analytics, and inspecting traffic.

## Accessing the Dashboard

1. Navigate to your frontend URL (e.g., `http://localhost:5173` for dev)
2. Log in with your email and password
3. You'll see the dashboard homepage with your tunnels

## Features Overview

The dashboard includes:

- 📊 **Dashboard** - Overview of active tunnels and stats
- 🚇 **Tunnels** - Manage all your tunnels
- 🔍 **Inspector** - View and replay HTTP requests
- 📈 **Analytics** - Charts and metrics
- 🔑 **API Keys** - Create and manage authentication keys
- ⚙️ **Settings** - Account and preferences

## Dashboard Page

The home page shows:

### Active Tunnels Card
- Number of currently active tunnels
- Tunnel status (online/offline)
- Quick actions (stop, restart)

### Traffic Stats
- Total requests today
- Bandwidth used
- Average latency
- Error rate

### Recent Activity
- Latest tunnel creation/deletion
- Recent requests
- Auth events

## Tunnels Management

### Viewing Tunnels

Navigate to **Tunnels** to see all your tunnels, both active and inactive.

Each tunnel card shows:
- **Subdomain/URL** - The public URL
- **Target** - Your local address (e.g., `localhost:3000`)
- **Protocol** - HTTP, HTTPS, or WebSocket
- **Status** - Active, inactive, or error
- **Created** - When the tunnel was first created
- **Last Active** - Most recent traffic timestamp

### Tunnel Actions

Click on a tunnel to see details and access actions:

- **Stop** - Close the tunnel connection
- **Delete** - Permanently remove the tunnel
- **View Inspector** - See traffic details
- **Share Link** - Copy tunnel URL to clipboard

### Creating Tunnels via UI

> [!NOTE]
> Tunnels are primarily created via SSH. The UI is for managing existing tunnels.

While you create tunnels via SSH commands, the dashboard shows them instantly after creation.

## Traffic Inspector

The Inspector is the most powerful feature for debugging.

### Accessing Inspector

1. Go to **Tunnels**
2. Click on a tunnel
3. Select **Inspector** tab

### Request List

The left panel shows all requests:
- **Method** (GET, POST, etc.)
- **Path** (e.g., `/api/users`)
- **Status** (200, 404, 500, etc.)
- **Latency** (response time in ms)
- **Timestamp** (when request occurred)

### Request Details

Click a request to see:

**Request Headers:**
```
Host: xyz123.yourdomain.com
User-Agent: Mozilla/5.0...
Accept: application/json
Authorization: Bearer ...
```

**Request Body:**
```json
{
  "username": "testuser",
  "email": "test@example.com"
}
```

**Response Headers:**
```
Content-Type: application/json
X-Powered-By: Express
Set-Cookie: session=...
```

**Response Body:**
```json
{
  "success": true,
  "id": "12345"
}
```

### Request Replay

Click **Replay** to resend the same request:
- Useful for debugging
- Modifies timestamp automatically
- Shows new response alongside original

### Filtering Requests

Filter by:
- **Method** - Only GET, POST, etc.
- **Status** - Only 2xx, 4xx, 5xx
- **Path** - Regex or exact match
- **Time Range** - Last hour, day, week

Example: Find all failed API requests:
```
Method: POST
Path: /api/*
Status: 5xx
```

## Analytics

### Overview Charts

**Requests Over Time**
- Line chart showing request volume
- Grouped by hour/day/week
- Filterable by tunnel

**Status Code Distribution**
- Pie chart of 2xx, 3xx, 4xx, 5xx responses
- Helps identify error patterns

**Latency Histogram**
- Distribution of response times
- Identify slow endpoints

**Bandwidth Usage**
- Upload and download over time
- Helps estimate costs

### Per-Tunnel Analytics

Click on a tunnel to see specific metrics:
- Total requests
- Average latency
- Error rate
- Unique IPs
- Top paths
- Top user agents

### Exporting Data

Click **Export** to download:
- CSV of all requests
- JSON dump of tunnel metadata
- Analytics summary PDF

## API Keys Management

### Viewing Keys

Navigate to **Settings → API Keys** to see all your keys:
- **Name** - Descriptive label
- **Prefix** - First few characters (e.g., `needle_abc123...`)
- **Created** - When key was generated
- **Last Used** - Most recent authentication
- **Expires** - Expiration date (if set)

### Creating a New Key

1. Click **Create API Key**
2. Enter a descriptive name (e.g., "Laptop Dev", "CI/CD")
3. Optionally set expiration date
4. Click **Generate**
5. **Copy the key immediately** - it won't be shown again!

Example key:
```
needle_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

### Key Security Best Practices

> [!CAUTION]
> **Treat API keys like passwords!**

- **One key per device/purpose** (laptop, CI, staging, etc.)
- **Set expiration dates** for temporary access
- **Rotate keys** every 90 days
- **Revoke immediately** if compromised

### Revoking Keys

If a key is compromised:
1. Find the key in the list
2. Click **Revoke**
3. Confirm revocation
4. The key becomes invalid immediately

All active tunnels using that key will disconnect.

## Settings

### Account

- **Email** - Your login email
- **Username** - Your unique username
- **Tier** - Free, Pro, or Enterprise
- **Created** - Account creation date

### Billing (Pro/Enterprise)

- **Current Plan** - Your subscription tier
- **Usage** - Current month's tunnel hours and bandwidth
- **Upgrade** - Change to higher tier
- **Downgrade** - Change to lower tier (at end of billing period)

### Preferences

- **Theme** - Dark Everblush (default) or Light
- **Timezone** - For analytics timestamps
- **Notifications** - Email alerts for tunnel errors
- **Language** - UI language (English default)

## Mobile Experience

The dashboard is responsive and works on mobile devices:
- **Tunnels list** - Stacked vertically
- **Inspector** - Swipe between request/response tabs
- **Charts** - Touch-optimized and scrollable

Access on mobile by visiting the dashboard URL from your phone's browser.

## Keyboard Shortcuts

Speed up your workflow:

| Shortcut | Action |
|----------|--------|
| `g d` | Go to Dashboard |
| `g t` | Go to Tunnels |
| `g i` | Go to Inspector |
| `g a` | Go to Analytics |
| `g s` | Go to Settings |
| `c` | Create API key |
| `?` | Show all shortcuts |

## Troubleshooting

### "Cannot connect to backend"

- Verify backend server is running
- Check `VITE_API_URL` matches your backend
- Check CORS configuration

### "Unauthorized" errors

- Log out and log back in
- Clear browser cache
- Your JWT may have expired

### Data not updating

- Hard refresh: `Ctrl+Shift+R` (or `Cmd+Shift+R` on Mac)
- Check WebSocket connection in browser DevTools

### Inspector showing old requests

- Click **Refresh** button
- Check time range filter
- Verify tunnel is still active

## Next Steps

- [Authentication](./authentication.md) - Learn about OAuth and login
- [API Reference](../developer-guide/api-reference.md) - Use the API directly
- [Troubleshooting](../operations/troubleshooting.md) - Solve common issues
