# Security Architecture

QuorumScope Engine is engineered with defense-in-depth principles tailored for blockchain monitoring infrastructure.

## Core Security Principles

### 1. Zero Private Key Access
The engine operates strictly as a read-only observer of public blockchain data. It never requires, stores, or processes secret signing keys or recovery phrases.

### 2. In-Memory Transaction Analysis
Candidate transactions submitted for preflight checks (`POST /api/v1/preflight`) are analyzed strictly in memory:
- No transaction envelopes are written to the database.
- No transaction payloads are persisted to disk or external caches.
- Server logs omit transaction bodies and decoded operations.

### 3. Request Bounds and Rate Limiting
- The preflight endpoint enforces a strict request body size limit of 256 KiB (`DefaultBodyLimit::max(262_144)`).
- Oversized payloads receive an immediate `413 Payload Too Large` error before parsing.

### 4. Database Sanitization
- All database queries use parameterized SQL via `sqlx` to protect against injection attacks.
- Internal database error messages are logged server-side and mapped to generic client error codes (`internal_error`) to prevent leaking internal database schemas or network configurations.

### 5. Cross-Origin Resource Sharing (CORS)
- CORS origin restrictions are enforced via Tower HTTP middleware.
- Unset or empty origin configurations disable CORS headers completely.
- Production and staging deployments explicitly whitelist the web console origin.

## Operational Auditing

For information on reporting vulnerabilities or reviewing disclosure policies, see [SECURITY.md](../SECURITY.md).
