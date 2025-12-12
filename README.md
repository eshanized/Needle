# Needle

A high-performance SSH tunneling service that exposes local applications to the internet through
memorable subdomains with automatic SSL termination.

Built with Rust for the backend and Vue.js for the management dashboard.

## Architecture

```
libneedle/     Rust backend  - SSH server, HTTP proxy, REST API
needleui/      Vue.js frontend - Dashboard, analytics, tunnel management
```

## Features

- Expose local apps via SSH tunnels with random or custom subdomains
- Real-time traffic inspection and request replay
- User authentication with OAuth and API keys
- Persistent tunnels that survive disconnections
- Per-tunnel rate limiting and abuse protection
- WebSocket support with transfer limits
- Management dashboard with live analytics

## Quick Start

### Prerequisites

- Rust 1.80+
- Node.js 20+
- Supabase project (for database and auth)
- Domain with wildcard DNS configured

### Backend

```sh
cd libneedle
cp .env.example .env
cargo run
```

### Frontend

```sh
cd needleui
npm install
npm run dev
```

## Configuration

All configuration is managed through environment variables. See `.env.example` in
each project directory for available options.

## License

MIT License - see [LICENSE](LICENSE) for details.
