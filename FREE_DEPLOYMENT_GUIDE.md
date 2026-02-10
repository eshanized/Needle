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

## 3. Backend Deployment (Oracle Cloud) - Detailed Guide

The backend needs to listen on port `2222` for SSH traffic. Most PaaS (Render/Heroku/Vercel) cannot do this on free tiers. Oracle Cloud "Always Free" provides a powerful ARM VM (Ampere) that can.

---

### Step 1: Sign Up for Oracle Cloud

1. Go to [Oracle Cloud Free Tier](https://www.oracle.com/cloud/free/)
2. Click **Start for free** and create an account
3. **Important**: You'll need a credit card for verification, but you will NOT be charged

---

### Step 2: Create a VM Instance

Once logged into Oracle Cloud Console:

1. **Navigate to Compute Instances**:
   - Click the **☰** hamburger menu (top left)
   - Go to **Compute** → **Instances**

2. **Click "Create instance"** button

3. **Configure the instance**:

   **Name**: Enter `needle-backend` (or any name you prefer)

   **Placement**:
   - Leave as default (usually your home region)

   **Image and shape**:
   - Click **"Change Image"**
   - Select **"Canonical Ubuntu"** from the list
   - Choose **Ubuntu 24.04 Minimal** or **Ubuntu 22.04 Minimal**
     - ⚠️ If you don't see 24.04, use 22.04 - both work perfectly
   - Click **"Select Image"**

   **Shape**:
   - Click **"Change Shape"**
   - Select **"Ampere"** (ARM processor - this is the free tier option)
   - Choose **VM.Standard.A1.Flex**
   - Set **OCPU count**: `4` (4 cores)
   - Memory will auto-adjust to **24 GB** (This is all free!)
   - Click **"Select Shape"**

   **Networking**:
   - Leave "Create new virtual cloud network" selected
   - Leave all defaults (it will create a VCN for you)
   - **✅ CRITICAL**: Make sure **"Assign a public IPv4 address"** is **CHECKED**

   **Add SSH keys**:
   - Select **"Generate SSH key pair"**
   - Click **"Save Private Key"** - Download and keep this safe!
   - Click **"Save Public Key"** (optional, but recommended as backup)

4. **Click "Create"** at the bottom

5. **Wait 2-3 minutes** for the instance to provision. Status will change from "PROVISIONING" to "RUNNING"

6. **Copy the Public IP Address** - You'll see it under "Instance Access" → "Public IP address"

---

### Step 3: Open Required Ports (Security Lists)

By default, Oracle Cloud blocks all ports except 22 (SSH). We need to open ports **3000** (API) and **2222** (SSH tunnels).

1. **Navigate to your VCN**:
   - From your instance page, scroll down to **"Primary VNIC"** section
   - Click the **VCN name** (usually starts with "vcn-")

2. **Go to Security Lists**:
   - On the left sidebar, under **"Resources"**, click **"Security Lists"**
   - Click the **"Default Security List for..."** link

3. **Add Ingress Rules**:
   - Click **"Add Ingress Rules"** button
   
   **Rule 1: API Port (3000)**
   - Source CIDR: `0.0.0.0/0`
   - IP Protocol: `TCP`
   - Source Port Range: (leave blank)
   - Destination Port Range: `3000`
   - Description: `Needle API`
   - Click **"Add Ingress Rules"**

   **Rule 2: SSH Tunnel Port (2222)**
   - Click **"Add Ingress Rules"** again
   - Source CIDR: `0.0.0.0/0`
   - IP Protocol: `TCP`
   - Destination Port Range: `2222`
   - Description: `Needle SSH Tunnels`
   - Click **"Add Ingress Rules"**

---

### Step 4: SSH Into Your Server

1. **Open your terminal** (Linux/Mac) or use **Git Bash/WSL** (Windows)

2. **Set key permissions** (Linux/Mac only):
   ```bash
   chmod 400 ~/Downloads/ssh-key-*.key
   ```

3. **Connect to the server**:
   ```bash
   ssh -i ~/Downloads/ssh-key-*.key ubuntu@<YOUR_PUBLIC_IP>
   ```
   Replace `<YOUR_PUBLIC_IP>` with the IP you copied earlier.

4. **Type "yes"** when asked if you want to continue connecting

---

### Step 5: Automated Setup (Recommended)

Once connected to your server, run this **one-liner**:

```bash
curl -fsSL https://raw.githubusercontent.com/eshanized/Needle/master/setup_oracle.sh | bash
```

**What this script does**:
1. Installs Docker and Docker Compose
2. Clones the Needle repository
3. Opens an editor for you to paste your Supabase credentials
4. Starts the backend

**During the script execution**:
- When the editor opens (nano), fill in:
  - `SUPABASE_URL` - Your Supabase project URL
  - `SUPABASE_ANON_KEY` - From Supabase Settings → API
  - `SUPABASE_SERVICE_ROLE_KEY` - From Supabase Settings → API (be careful, this is secret!)
  - `JWT_SECRET` - Generate with: `openssl rand -hex 32`
  - `DOMAIN` - Your Public IP or domain name
- Press `Ctrl+X`, then `Y`, then `Enter` to save

---

### Step 6: Verify Deployment

After the script completes, check if the backend is running:

```bash
curl http://localhost:3000/health
```

You should see: `{"status":"ok"}`

**Your backend is now live at**:
- API: `http://<YOUR_PUBLIC_IP>:3000`
- SSH Tunnel Server: `<YOUR_PUBLIC_IP>:2222`

---

### (Optional) Manual Setup

If you prefer to do it manually instead of using the script:

```bash
# 1. Install Docker
sudo apt update
sudo apt install -y docker.io docker-compose-v2 git
sudo usermod -aG docker $USER
newgrp docker

# 2. Clone Repository
git clone https://github.com/eshanized/Needle.git
cd Needle

# 3. Configure Environment
cp libneedle/.env.example libneedle/.env
nano libneedle/.env
# Fill in your Supabase credentials

# 4. Start Backend (API service only, since UI is on Vercel)
docker compose up -d --build api
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
