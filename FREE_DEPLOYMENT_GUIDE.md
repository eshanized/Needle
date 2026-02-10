# Needle Free Deployment Guide

This guide explains how to deploy the full Needle stack (Frontend, Backend, Database) for free.

## Architecture & Hosting Strategy

| Component | Technology | Free Hosting Service | Why? |
|-----------|------------|----------------------|------|
| **Frontend** | Vue.js + Vite | **Vercel** | Best for SPAs, free SSL, global CDN, separates UI from Docs. |
| **Backend** | Rust (Axum + SSH) | **Oracle Cloud Always Free** | The ONLY free tier robust enough to allow opening custom ports (2222 for SSH) and persistent TCP connections. |
| **Database** | PostgreSQL | **Supabase** | Generous free tier (500MB), easy setup, managed backups. |
| **Docs** | MkDocs | **GitHub Pages** | Already set up! |

---

## 1. Database (Supabase)

You should already have this from development.

1. Go to [Supabase](https://supabase.com/) and create a free project.
2. In Project Settings -> API, note down:
   - Project URL (`SUPABASE_URL`)
   - `anon` public key (`SUPABASE_ANON_KEY`)
   - `service_role` secret (`SUPABASE_SERVICE_ROLE_KEY`)
3. Go to SQL Editor and run the contents of `libneedle/schema.sql`.

---

## 2. Frontend Deployment (Vercel)

Vercel is the gold standard for deploying Vue.js applications.

### Setup (One-Time)
1. Push your code to GitHub.
2. Sign up at [Vercel](https://vercel.com/signup) using GitHub.
3. Click "Add New..." -> "Project".
4. Import your `Needle` repository.

### Configuration
1. **Framework Preset**: Vite
2. **Root Directory**: Click "Edit" and select `needleui` folder.
3. **Environment Variables**:
   Add the following variables (Vercel handles `VITE_` prefix automatically for exposure):
   - `VITE_API_URL`: The URL of your backend (e.g., `https://api.yourdomain.com` or IP)
   - `VITE_SUPABASE_URL`: Your Supabase URL
   - `VITE_SUPABASE_ANON_KEY`: Your Supabase Anon Key

4. Click **Deploy**.

### Custom Domain (Optional)
Vercel gives you a `needle-frontend.vercel.app` domain. You can add your own in Settings -> Domains.

---

## 3. Backend Deployment (Oracle Cloud)

The backend needs to listen on port `2222` for SSH traffic. Most PaaS (Render/Heroku/Vercel) cannot do this on free tiers. Oracle Cloud "Always Free" provides a powerful ARM VM (Ampere) that can.

### A. Get the Server
1. Sign up for [Oracle Cloud Free Tier](https://www.oracle.com/cloud/free/).
2. Create a **VM instance**:
   - **Image**: **Canonical Ubuntu 24.04** or **22.04** Minimal (Always pick the "Minimal" or standard version, avoid old versions).
   - **Shape**: **VM.Standard.A1.Flex** (Ampere ARM). Select 4 OCPU and 24GB RAM (Yes, it's free!).
3. Download the SSH key provided.
4. Note the **Public IP Address**.

### B. Configure Network & Ports
1. In Oracle Cloud Console, go to **Virtual Cloud Network (VCN)** -> **Security Lists**.
2. Edit the Default Security List.
3. Add **Ingress Rules**:
   - **Source**: `0.0.0.0/0`
   - **Protocol**: TCP
   - **Destination Port Range**: `3000` (API), `2222` (SSH), `80`, `443`.

### C. Deploy the Code
SSH into your new server:
`ssh -i ssh-key-202X.key ubuntu@<YOUR_PUBLIC_IP>`

Run these commands on the server:

```bash
# 1. Update and install Docker
sudo apt update
sudo apt install -y docker.io docker-compose
sudo usermod -aG docker $USER
newgrp docker

# 2. Clone Repository
git clone https://github.com/eshanized/Needle.git
cd Needle

# 3. Configure Environment
# Create .env from example and fill in your Supabase keys
cp libneedle/.env.example libneedle/.env
nano libneedle/.env
# EDIT: Set API_ADDR=0.0.0.0:3000, SSH_ADDR=0.0.0.0:2222, DOMAIN=<YOUR_IP_OR_DOMAIN>

# 4. Run with Docker Compose
# (We need to update compose file to bind ports to host)
docker compose up -d --build
```

### D. DNS (Optional but recommended)
1. Buy a cheap domain (e.g. from Namecheap).
2. Create **A Records**:
   - `@` -> Your Oracle IP
   - `*` -> Your Oracle IP (Wildcard for subdomains)

---

## Alternative: Fly.io (Low Cost)

If Oracle Cloud is too complex, **Fly.io** is much easier but costs ~$2/mo for a dedicated IPv4 address (required for SSH on port 2222).

1. Install Fly CLI: `curl -L https://fly.io/install.sh | sh`
2. `fly launch` in `libneedle/` directory.
3. Edit `fly.toml` to expose port 2222 (TCP) and 443 (HTTP).
4. `fly ips allocate-v4` (Costs money!)
5. `fly deploy`

---

---

## FAQ: Why not Render, Heroku, or Railway?

You might prefer standard PaaS providers like Render, but they are **not suitable** for Needle's backend for two critical reasons:

1.  **Blocker: TCP Ports**: Needle requires **two** open ports:
    *   Port 3000 (HTTP) for the API
    *   Port 2222 (SSH) for tunnels
    Most free PaaS (Render, Heroku) only expose a **single HTTP port** (80/443). They do not support raw TCP traffic on arbitrary ports, which means your SSH tunnels would never connect.

2.  **Blocker: State Sharing**: Needle stores tunnel state in memory (for speed). This requires the API and SSH server to run in the *same process*. You cannot deploy them as separate services on Render because they wouldn't share the tunnel state.

**Oracle Cloud Always Free** is unique because it gives you a **Virtual Machine (VM)**. You control the firewall rules and can open any ports you want (like 2222), making it the only viable free option.

## Summary

- **Frontend**: https://needle-ui.vercel.app (Free)
- **Backend API**: http://<ORACLE_IP>:3000 (Free)
- **Tunnel Server**: <ORACLE_IP>:2222 (Free)
- **Docs**: https://eshanized.github.io/Needle (Free)
