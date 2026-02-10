#!/bin/bash
# setup_oracle.sh - Automated deployment script for Needle Backend on Oracle Cloud (Ubuntu)

set -e

echo "🚀 Starting Needle Backend Setup..."

# 1. Update System & Install Dependencies
echo "📦 Updating system and installing dependencies..."
sudo apt-get update
sudo apt-get install -y ca-certificates curl gnupg git

# 2. Install Docker (Official Script)
if ! command -v docker &> /dev/null; then
    echo "🐳 Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    sudo usermod -aG docker $USER
    echo "✅ Docker installed. Please log out and back in for group changes to take effect, or run 'newgrp docker' now."
else
    echo "✅ Docker is already installed."
fi

# 3. Clone Repository (if not already present)
if [ -d "Needle" ]; then
    echo "📂 Needle directory exists, pulling latest changes..."
    cd Needle
    git pull
else
    echo "📂 Cloning Needle repository..."
    git clone https://github.com/eshanized/Needle.git
    cd Needle
fi

# 4. Configure Environment
if [ ! -f "libneedle/.env" ]; then
    echo "⚙️ Configuring environment variables..."
    cp libneedle/.env.example libneedle/.env
    
    echo ""
    echo "⚠️  IMPORTANT: You need to edit libneedle/.env with your Supabase credentials!"
    echo "   Running 'nano libneedle/.env' for you now..."
    read -p "   Press Enter to open editor..."
    nano libneedle/.env
else
    echo "✅ libneedle/.env already exists."
fi

# 5. Build and Run Backend Only
echo "🚀 Building and starting Needle Backend..."
# We only start the 'api' service, effectively ignoring the 'ui' service since frontend is on Vercel
docker compose up -d --build api

echo ""
echo "✅ Deployment Complete!"
echo "---------------------------------------------------"
echo "📡 Backend API: http://$(curl -s ifconfig.me):3000"
echo "🚇 SSH Tunnel:  $(curl -s ifconfig.me):2222"
echo "---------------------------------------------------"
echo "To view logs: docker compose logs -f api"
