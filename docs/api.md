# HTTP API

Run `quorumscope serve --bind 127.0.0.1:8080` with `DATABASE_URL` set to the PostgreSQL database. The server applies migrations at startup. The generated contract is [openapi.json](../openapi/openapi.json), and the server also exposes it at `GET /openapi.json`.

All state endpoints read PostgreSQL. The optional `network_id` query parameter selects a network UUID. Without it, the first network by name is used. A missing network returns `404`. List endpoints accept one-based `page` (default `1`) and `page_size` (default `50`, maximum `100`). Results are ordered and returned as `{ "items": [...], "page": 1, "page_size": 50 }`. An empty page has an empty `items` array.

| Endpoint | Response |
| --- | --- |
| `GET /api/v1/network` | Selected network ID, name, and passphrase |
| `GET /api/v1/freeze-state` | Latest checkpoint ledger and counts of active frozen keys, bypasses, and incidents |
| `GET /api/v1/frozen-keys` | Paginated active frozen keys |
| `GET /api/v1/frozen-keys/{id}` | Active frozen key by 64-character hex hash |
| `GET /api/v1/bypasses` | Paginated active bypasses |
| `GET /api/v1/incidents` | Paginated incidents |
| `GET /api/v1/incidents/{id}` | Incident by UUID |
| `GET /api/v1/incidents/{id}/timeline` | Paginated incident events |
| `GET /api/v1/status` | Most recently updated indexer checkpoint for the selected network; `404` if none exists |
| `GET /health/live` | Process liveness |
| `GET /health/ready` | Database readiness (`503` when unavailable) |

Errors use `{ "error": { "code": "...", "message": "..." }, "request_id": "uuid" }`. The same UUID is returned in the `x-request-id` response header. Invalid IDs and pagination return `400`; missing resources return `404`; storage read failures return `500` with database details kept in server logs.

Regenerate the committed schema with `cargo run -p quorumscope-api --example openapi`. A test fails if the generated schema differs from the committed artifact.
