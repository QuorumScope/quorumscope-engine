# Deployment Guide

This guide covers deployment options for QuorumScope Engine, including local containerized environments, cloud hosting, and staging infrastructure.

## Staging Deployment

The QuorumScope staging environment is configured as follows:

| Component | Platform | Configuration |
| --- | --- | --- |
| API Server | Render | Rust binary container service |
| Database | Supabase | Managed PostgreSQL 16 |
| Indexer Target | Stellar Testnet | `https://soroban-testnet.stellar.org` |
| Public API URL | Render | `https://quorumscope-engine-api.onrender.com` |
| OpenAPI Spec | Render | `https://quorumscope-engine-api.onrender.com/openapi.json` |

### Render Staging Details

The staging service is defined in `render.yaml` and started using `scripts/render-start.sh`.
- **Free Tier Hibernation**: Render free tier services suspend after 15 minutes of inactivity. When a new request arrives, a cold start of 30 to 60 seconds may occur.
- **Freshness Upon Wake**: Because indexing only executes when the service is active, the first status check upon waking may show a lag or stale status until the polling cycle refreshes the state against the testnet RPC.
- **CORS Configuration**: The staging API sets `ALLOWED_ORIGINS="https://quorumscope-console.vercel.app"` to permit browser preflight checks from the staging console.

## Local Docker Deployment

The repository includes a `docker-compose.yml` file for running PostgreSQL locally.

```bash
# Start local PostgreSQL
make db-up

# Set database URL
export DATABASE_URL="postgres://quorumscope:quorumscope_dev@localhost:5432/quorumscope"

# Apply migrations
cargo run -p quorumscope-cli -- init

# Run API server
cargo run -p quorumscope-cli -- serve --bind 127.0.0.1:8080
```

## Production Deployment Recommendations

For production or persistent monitoring deployments:
1. **Always-On Worker**: Run `quorumscope index watch` in a dedicated background worker process with process supervision (systemd, Kubernetes Deployment, or ECS task).
2. **Dedicated Database**: Use a high-availability PostgreSQL 16+ instance with automated backups and sufficient connection pool capacity (`max_connections >= 50`).
3. **CORS Hardening**: Explicitly specify the web console origin in `ALLOWED_ORIGINS`. Never leave CORS open to wildcards.
4. **Health Probes**:
   - Liveness probe: `GET /health/live` (checks HTTP server responsiveness)
   - Readiness probe: `GET /health/ready` (checks database connection pool health)
5. **Reverse Proxy & TLS**: Terminate TLS at a reverse proxy (e.g., Cloudflare, NGINX, or cloud load balancer) in front of the API server.
