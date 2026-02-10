# FAQ

## General

### What is Needle?

Needle is a self-hosted SSH tunneling service that exposes local applications to the internet through memorable subdomains with automatic SSL termination.

### How is Needle different from ngrok?

- **Self-hosted** - You control the infrastructure
- **Open source** - MIT licensed
- **Configurable** - Adjust limits, timeouts, tiers as needed
- **Free** - No subscription required (though you pay for hosting)

### Do I need to be an expert to use Needle?

No! If you can run `ssh` and have basic Linux knowledge, you can use Needle. Deploying Needle requires more expertise, but using it is straightforward.

## Features

### Can I use custom subdomains?

Yes, with a Pro or Enterprise tier subscription. Free tier gets random subdomains.

### Does Needle support WebSockets?

Yes! WebSocket connections are automatically detected and proxied.

### Can I inspect HTTP traffic?

Yes! The web dashboard has a traffic inspector that shows request/response headers, bodies, status codes, and latency.

### Is HTTPS automatic?

Yes, all tunnel URLs use HTTPS with automatic SSL termination at the server.

## Technical

### What ports can I forward?

By default, you can forward ports >= 1024 (non-privileged). This is configurable by the administrator via `MIN_SSH_PORT`.

### How many concurrent tunnels can I have?

Depends on your tier:
- **Free**: 3 concurrent tunnels
- **Pro**: 50 concurrent tunnels
- **Enterprise**: 500 concurrent tunnels

### How long do tunnels stay active?

Tunnels stay active as long as your SSH connection is alive. Pro/Enterprise tiers support persistent tunnels that survive disconnections.

### What happens if my local app crashes?

The tunnel remains active, but requests will fail since there's nothing to forward to. You'll see connection errors in the inspector.

### Can I share a tunnel URL with others?

Yes! Tunnel URLs are public. Anyone with the URL can access your local app.

## Troubleshooting

### My tunnel works but I get SSL errors

Verify your wildcard SSL certificate includes all subdomains:

```bash
sudo certbot certificates
# Should show *.yourdomain.com
```

### I'm getting "rate limit exceeded"

Wait a few minutes for the rate limit to reset. If you consistently hit limits, consider:
- Upgrading tier (higher limits)
- Contacting admin to increase per-IP limits

### How do I debug tunnel connection issues?

1. Check SSH connection: `ssh -v tunnel@yourdomain.com -p 2222`
2. Check local app is running: `curl http://localhost:YOUR_PORT`
3. Check Needle logs: `journalctl -u needle -f`
4. Check firewall rules
5. Verify DNS resolves correctly

## Security

### Is my traffic encrypted?

Yes! All tunnel traffic uses HTTPS (TLS 1.2+). The SSH tunnel itself is also encrypted end-to-end.

### Can Needle see my traffic?

The server can technically see traffic since it terminates SSL. However, request/response bodies are only stored if you enable traffic inspection, and only you can view them (enforced via Row-Level Security).

### What if my API key is compromised?

Revoke the key immediately from the dashboard. All active tunnels using that key will disconnect. Create a new key to replace it.

### Should I use Needle for production traffic?

Needle is designed for development and testing. For production, use a proper deployment with load balancers, not ad-hoc tunnels.

## Billing & Tiers

### Is Needle free?

Needle is open source (MIT license), so the software is free. However:
- You pay for hosting (server, database)
- Pro/Enterprise tiers are conceptually "paid features" (you configure tier limits)
- There's no payment processing in Needle itself (you implement that)

### How do I upgrade my tier?

Contact your Needle administrator. Tier assignment is manual in the database (`users.tier` column).

## Deployment

### Can I run Needle on my laptop?

Yes! For personal use or development. But for sharing with others, you'll need:
- A public server
- A domain name
- SSL certificates

### What are the minimum server requirements?

- 2GB RAM (4GB recommended)
- 2 CPU cores (4+ recommended)
- Ubuntu 22.04 LTS or similar
- Public IP address

### Can I use Docker?

Docker support is not officially provided, but you can create your own `Dockerfile`:

```dockerfile
FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/needle-server /usr/local/bin/
CMD ["needle-server"]
```

## Contributing

### Can I contribute to Needle?

Absolutely! See the [Contributing guide](../developer-guide/contributing.md).

### I found a bug. What should I do?

Open an issue on GitHub with:
- Steps to reproduce
- Expected vs actual behavior
- Needle version and OS
- Relevant logs (without secrets!)

### I have a feature request

Open a GitHub issue describing:
- The use case
- Why existing features don't solve it
- Your proposed solution (optional)

## Still have questions?

- **Documentation**: This site!
- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions in GitHub Discussions
- **Email**: Contact the maintainer (see README)
