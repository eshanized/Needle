# Quick Start

Get up and running with Needle in under 5 minutes!

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.80+** - [Install from rustup.rs](https://rustup.rs/)
- **Node.js 20+** - [Download from nodejs.org](https://nodejs.org/)
- **Supabase Account** - [Sign up at supabase.com](https://supabase.com/)
- **Domain with Wildcard DNS** - Configure `*.yourdomain.com` → your server IP

## Step 1: Clone the Repository

```bash
git clone https://github.com/eshanized/needle.git
cd needle
```

## Step 2: Set Up the Database

1. Create a new Supabase project
2. Copy your project credentials
3. Run the schema script:

```bash
cd libneedle
psql "$SUPABASE_DATABASE_URL" < schema.sql
```

## Step 3: Configure Environment Variables

Create `libneedle/.env`:

```bash
# Database
SUPABASE_URL=https://xxxxx.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Security (generate with: openssl rand -hex 32)
JWT_SECRET=your-256-bit-secret

# Server
API_ADDR=0.0.0.0:3000
SSH_ADDR=0.0.0.0:2222
DOMAIN=yourdomain.com
```

## Step 4: Start the Backend

```bash
cd libneedle
cargo run --bin needle-server
```

You should see:

```
INFO needle_server: Starting Needle server
INFO needle_server: API server listening on 0.0.0.0:3000
INFO needle_server: SSH server listening on 0.0.0.0:2222
```

## Step 5: Start the Frontend (Optional)

In a new terminal:

```bash
cd needleui
npm install
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

## Step 6: Create Your First User

Register through the web UI or use the API:

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "you@example.com",
    "username": "yourname",
    "password": "secure-password"
  }'
```

## Step 7: Create an API Key

Log in to the dashboard and navigate to **Settings → API Keys** → **Create New Key**.

Save your key - it will look like: `needle_xxxxxxxxxxxxxxxx`

## Step 8: Create Your First Tunnel

Using SSH port forwarding:

```bash
ssh -R 80:localhost:3000 tunnel@yourdomain.com -p 2222 \
  -o "User=needle_xxxxxxxxxxxxxxxx"
```

Replace:
- `localhost:3000` - Your local app's address
- `yourdomain.com` - Your Needle server domain
- `needle_xxxxxxxxxxxxxxxx` - Your API key

You'll see output like:

```
Tunnel created: https://abc123.yourdomain.com → localhost:3000
```

## Step 9: Test Your Tunnel

Visit `https://abc123.yourdomain.com` in your browser. You should see your local app!

## Step 10: View Traffic in the Dashboard

Go to the dashboard and click on your tunnel. You'll see:
- Active connections
- Request history
- Real-time traffic
- Analytics charts

---

## Common Commands

### Create a Random Tunnel
```bash
ssh -R 80:localhost:8080 tunnel@yourdomain.com -p 2222 \
  -o "User=needle_yourApiKey"
```

### Create a Custom Subdomain (Pro Tier)
```bash
ssh -R myapp:80:localhost:8080 tunnel@yourdomain.com -p 2222 \
  -o "User=needle_yourApiKey"
```

### Create a WebSocket Tunnel
```bash
ssh -R 80:localhost:9000 tunnel@yourdomain.com -p 2222 \
  -o "User=needle_yourApiKey"
```

### Check Health
```bash
curl http://localhost:3000/health
```

### View Metrics
```bash
curl http://localhost:3000/metrics
```

---

## Troubleshooting

### "Connection refused" when creating tunnel
- Ensure the Needle server is running (`cargo run`)
- Check that SSH_ADDR is accessible from your client

### "Authentication failed"
- Verify your API key is correct
- Check that the key hasn't expired in the dashboard

### "Database connection error"
- Confirm SUPABASE_URL and keys are correct
- Verify the database schema was applied

### Tunnel created but can't access the URL
- Check DNS is configured correctly (wildcard A record)
- Verify your local app is actually running
- Check firewall rules allow traffic on port 3000

---

## Next Steps

- **Configuration**: Learn about all [configuration options](./user-guide/configuration.md)
- **Dashboard**: Explore the [web dashboard features](./user-guide/dashboard.md)
- **API**: Read the full [API reference](./developer-guide/api-reference.md)
- **Deployment**: Set up [production deployment](./operations/deployment.md)

Need help? Check the [FAQ](./appendix/faq.md) or open an issue on GitHub!
