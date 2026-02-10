# Glossary

## A

**API Key**  
A long-lived authentication token used for SSH tunnel authentication. Format: `needle_` followed by 32 random characters.

**Argon2**  
Password hashing algorithm used to securely store user passwords.

**Async**  
Asynchronous programming pattern used throughout Needle's Rust backend via Tokio runtime.

**Axum**  
Web framework for Rust used to build Needle's REST API.

## C

**CORS (Cross-Origin Resource Sharing)**  
Security mechanism that controls which domains can access the API from browsers.

**Custom Subdomain**  
User-chosen subdomain (e.g., `myapp.yourdomain.com`) instead of random. Requires Pro or Enterprise tier.

## H

**HTTP Proxy**  
Needle component that forwards HTTP/HTTPS traffic from tunnel URLs to local applications.

## J

**JWT (JSON Web Token)**  
Authentication token format used for API access. Contains user ID, email, tier, and expiration.

## M

**mdBook**  
Documentation generator that creates this site from Markdown files.

**MVCC (Multi-Version Concurrency Control)**  
Database technique used by PostgreSQL to handle concurrent transactions.

## P

**Persistent Tunnel**  
Tunnel configuration that survives SSH disconnections (Pro/Enterprise feature).

**Prometheus**  
Monitoring system that collects metrics from Needle's `/metrics` endpoint.

## R

**Rate Limiting**  
Mechanism to prevent abuse by limiting requests per time period (e.g., 5 logins/minute).

**RLS (Row-Level Security)**  
PostgreSQL feature that ensures users can only access their own data, enforced at database level.

**russh**  
Rust library implementing the SSH protocol, used for Needle's SSH server.

## S

**SSH (Secure Shell)**  
Protocol used to create encrypted tunnels between your machine and Needle server.

**SSH Tunnel**  
Encrypted connection that forwards traffic from a public URL to your local application.

**Subdomain**  
Prefix before your domain (e.g., `abc123` in `abc123.yourdomain.com`).

**Supabase**  
Hosted PostgreSQL database service with built-in authentication and Row-Level Security.

## T

**Tier**  
Subscription level determining tunnel limits: Free (3tunnels), Pro (50), Enterprise (500).

**Tokio**  
Asynchronous runtime for Rust that powers Needle's backend.

**Tunnel**  
A mapping from a public subdomain to your local application endpoint.

**Tunnel Manager**  
Needle component responsible for creating, tracking, and destroying tunnels.

## W

**WebSocket**  
Bi-directional communication protocol supported by Needle for real-time applications.

**Wildcard DNS**  
DNS configuration (e.g., `*.yourdomain.com`) that routes all subdomains to your server.

## Acronyms

| Acronym | Full Form |
|---------|-----------|
| API | Application Programming Interface |
| CI/CD | Continuous Integration / Continuous Deployment |
| CORS | Cross-Origin Resource Sharing |
| DNS | Domain Name System |
| HTTPS | Hypertext Transfer Protocol Secure |
| JWT | JSON Web Token |
| MVCC | Multi-Version Concurrency Control |
| RLS | Row-Level Security |
| SSH | Secure Shell |
| SSL | Secure Sockets Layer |
| TLS | Transport Layer Security |
| TTL | Time To Live |
| URL | Uniform Resource Locator |
| UUID | Universally Unique Identifier |
