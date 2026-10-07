# QuorumScope Engine Documentation

QuorumScope Engine is the backend protocol processing and indexing layer for Stellar Quorum Freeze (CAP-77). It extracts freeze state and bypass configurations from Stellar RPC nodes, maintains state change records with cryptographic evidence in PostgreSQL, and serves developer APIs for preflight simulation and impact analysis.

## Documentation Index

- [Architecture](architecture.md): System components, data pipelines, and storage models.
- [API Contract](api.md): REST endpoints, freshness payloads, pagination, and error envelopes.
- [Indexer](indexer.md): Poll-and-reconcile synchronization, checkpoints, and change tracking.
- [Preflight Analysis](preflight.md): Transaction envelope simulation across accounts, trustlines, and Soroban contracts.
- [Data Freshness](data-freshness.md): Freshness statuses, ledger lag definitions, and compatibility ratings.
- [Deployment](deployment.md): Running the engine locally, with Docker, or on cloud platforms like Render.
- [Testing & Verification](testing.md): Test suites, wiremock fixtures, PostgreSQL integration tests, and verification boundaries.
- [Verification Status](verification.md): Detailed verification record against live testnet and synthetic models.
- [Security](security.md): Security posture, data privacy policies, and non-custodial operations.
- [Known Limitations](limitations.md): Current protocol, storage, and infrastructure boundaries.
- [Protocol Versions](versions.md): Tracking verified protocol versions and upgrade paths.
- [Release Notes](release-notes.md): Notes for `v0.1.0-staging`.

## Staging Endpoints

- Staging API: [https://quorumscope-engine-api.onrender.com](https://quorumscope-engine-api.onrender.com)
- OpenAPI Specification: [https://quorumscope-engine-api.onrender.com/openapi.json](https://quorumscope-engine-api.onrender.com/openapi.json)
- Web Console: [https://quorumscope-console.vercel.app](https://quorumscope-console.vercel.app)

## Quick CLI Reference

```bash
# Apply database migrations
quorumscope init

# Run a one-time index sync
quorumscope index once

# Poll continuously for freeze changes
quorumscope index watch

# Serve HTTP API on port 8080
quorumscope serve --bind 127.0.0.1:8080

# Generate a PDF incident report
quorumscope report --incident <UUID> --output report.pdf
```
