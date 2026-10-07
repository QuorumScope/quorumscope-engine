# Release Notes

## Release v0.1.0-staging (2026-10-07)

QuorumScope Engine `v0.1.0-staging` marks the initial staging deployment of the Stellar Quorum Freeze indexing and preflight service.

### Summary of Deliverables

- **CAP-77 RPC Ingestion**: Ingests `FrozenLedgerKeys` and `FreezeBypassTxs` config entries from Stellar RPC nodes.
- **Persistent Storage**: PostgreSQL schema managing networks, checkpoints, freeze history, bypass transactions, and incident episodes.
- **Axum API Server**: Read endpoints for network status, keys, bypasses, episodes, and impact, plus in-memory preflight simulation.
- **CLI Commands**: `init`, `serve`, `index once`, `index watch`, and `report`.
- **OpenAPI 3.0 Contract**: Published at `GET /openapi.json`.
- **Typst Incident Reports**: Automatic compilation of PDF incident summaries.

### Honest Status & Constraints

- **Staging URL**: [https://quorumscope-engine-api.onrender.com](https://quorumscope-engine-api.onrender.com)
- **Target Network**: Stellar Testnet (`https://soroban-testnet.stellar.org`)
- **Protocol 28 Verified Max**: `VERIFIED_PROTOCOL_MAX` is set to 28. Because testnet reports protocol 29, the API correctly flags `unverified_protocol`.
- **Empty Freeze Set**: Testnet currently holds 0 frozen keys and 0 bypasses. Non-empty freeze set behavior is verified via mocked RPC tests and fixtures.
- **Render Free Tier**: The hosting service spins down during idle periods, causing cold-start latency on initial requests.
- **Production Readiness**: This release is intended for staging evaluation, integration testing, and developer feedback. It is not an enterprise SLA production service.
