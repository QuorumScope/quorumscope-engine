# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Verification of Protocol 29 compatibility once testnet stabilization occurs.
- Continuous indexing deployment on dedicated hosting infrastructure.
- Ingestion of transaction history to populate `recently_observed` and `dependency_observed` impact classes.

## [v0.1.0-staging] - 2026-10-07

### Added
- Stellar RPC ingestion service polling `FrozenLedgerKeys` and `FreezeBypassTxs` config setting entries via `getLedgerEntries`.
- PostgreSQL storage schema with migrations covering networks, checkpoints, ledger keys, freeze state changes, bypasses, episodes, impact, and evidence.
- Full Axum HTTP API serving `/health/live`, `/health/ready`, `/api/v1/network`, `/api/v1/status`, `/api/v1/freeze-state`, `/api/v1/frozen-keys`, `/api/v1/bypasses`, `/api/v1/incidents`, `/api/v1/impact`, and `/api/v1/preflight`.
- Interactive preflight simulation analyzing transaction envelope XDR across account, trustline, claimable balance, liquidity pool, and Soroban contract keys.
- OpenAPI 3.0 specification endpoint at `GET /openapi.json`.
- Command-line interface with subcommands: `init`, `serve`, `index once`, `index watch`, and `report`.
- Incident report generation producing PDF files via Typst.
- Automatic request ID generation (`x-request-id`) and sanitized client error envelopes.
- Configurable CORS origin filtering via `ALLOWED_ORIGINS`.
- Prometheus metrics endpoint at `GET /metrics`.
- Staging deployment on Render: [Staging API](https://quorumscope-engine-api.onrender.com).
- Staging OpenAPI definition: [OpenAPI JSON](https://quorumscope-engine-api.onrender.com/openapi.json).

### Changed
- Standardized `VERIFIED_PROTOCOL_MAX` default to 28 across CLI flags and environment variables.
- Refined freeze episode reconciliation to handle out-of-order polls and database recovery.

### Fixed
- Enforced body size limits on `POST /api/v1/preflight` to prevent unbounded memory allocation from malformed payloads.
- Corrected status lag calculations when the indexer trails the latest network ledger.

### Known limitations
- **No live non-empty freeze set observed**: Stellar Testnet currently contains 0 frozen keys. Non-empty freeze states and bypass resolutions are validated through wiremock RPC tests and fixtures.
- **Protocol 29 unverified warning**: Stellar Testnet reports Protocol 29, which exceeds `VERIFIED_PROTOCOL_MAX=28`. The API correctly returns `unverified_protocol` status.
- **Render free tier sleep**: The staging API sleeps after inactivity. Cold starts can take up to 60 seconds, during which data freshness may temporarily read as stale until the next poll cycle.
- **Uncollected impact classes**: Impact classes requiring transaction history (`recently_observed`, `dependency_observed`, `inferred`) are not collected.
- **Ledger close times**: Ledger close timestamps are not persisted.
- **Audit status**: The codebase has not undergone a formal third-party security audit.
