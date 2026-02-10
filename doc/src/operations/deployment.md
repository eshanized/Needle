# Deployment

Production deployment guide for Needle.

[_This content is based on the existing DEPLOY.md file_]

## Quick Start

See the [official DEPLOY.md](../../DEPLOY.md) for the most up-to-date deployment instructions.

## Pre-Production Checklist

Before deploying to production:

- [ ] Review [Production Certification](./production-certification.md)
- [ ] Complete [Security Audit](./security.md)
- [ ] Set up [Monitoring](./monitoring.md)
- [ ] Configure backups
- [ ] Test disaster recovery procedures

## Environment Setup

### Server Requirements

- **OS**: Linux (Ubuntu 22.04 LTS recommended)
- **RAM**: 4GB minimum, 8GB recommended
- **CPU**: 2 cores minimum, 4+ recommended
- **Disk**: 20GB minimum for logs and database
- **Network**: Public IPv4 address

### DNS Configuration

Configure wildcard DNS:

```
Type: A
Host: *.yourdomain.com
Points to: YOUR_SERVER_IP
TTL: 300 (5 minutes)
```

Verify:
```bash
dig abc123.yourdomain.com
# Should return YOUR_SERVER_IP
```

## SystemD Service

Create `etc/systemd/system/needle.service`:

```ini
[Unit]
Description=Needle Tunneling Service
After=network.target

[Service]
Type=simple
User=needle
Group=needle
WorkingDirectory=/opt/needle
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

## Reverse Proxy with Nginx

Install Nginx and Certbot:

```bash
sudo apt update
sudo apt install nginx certbot python3-certbot-nginx
```

Configure Nginx (`/etc/nginx/sites-available/needle`):

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
        deny all;  # Restrict metrics to monitoring system only
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

Enable site:

```bash
sudo ln -s /etc/nginx/sites-available/needle /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

Get SSL certificates:

```bash
sudo certbot --nginx -d yourdomain.com -d *.yourdomain.com
```

## Firewall

```bash
# Allow HTTPS
sudo ufw allow 443/tcp

# Allow SSH tunneling
sudo ufw allow 2222/tcp

# Deny direct API access (use nginx proxy)
sudo ufw deny 3000/tcp

# Enable firewall
sudo ufw enable
```

## Upgrading

```bash
# 1. Build new version
cd /opt/needle
git pull
cargo build --release

# 2. Stop service
sudo systemctl stop needle

# 3. Backup database (if schema changed)
pg_dump $DATABASE_URL > backup-$(date +%Y%m%d).sql

# 4. Run migrations
psql $DATABASE_URL < migrations/YYYYMMDD.sql

# 5. Start service
sudo systemctl start needle

# 6. Verify
curl http://localhost:3000/health
```

## Rolling Back

```bash
sudo systemctl stop needle
cp bin/needle-server.backup bin/needle-server
psql $DATABASE_URL < backup-YYYYMMDD.sql
sudo systemctl start needle
```

## Next Steps

- [Monitoring](./monitoring.md) - Set up Prometheus and Grafana
- [Security](./security.md) - Harden your deployment
- [Troubleshooting](./troubleshooting.md) - Common issues
