# Release Notes

## Release v0.1.1-staging (2026-10-28)

This hardening release verifies the CAP-77 compatibility surface against Stellar Protocol 29 and adds a reproducible non-empty fixture demo for reviewer and contributor workflows.

### What changed

- Verified QuorumScope's current CAP-77 XDR/config-entry, RPC, indexing, and preflight compatibility against Protocol 29.
- Added a deterministic fixture demo that runs the real indexer, PostgreSQL storage, API, and preflight path with frozen keys and bypass state.
- Added Protocol 30 readiness boundaries for the upcoming, not-yet-active Bell upgrade.
- Added Dependabot configuration for Cargo and GitHub Actions.
- Aligned OpenAPI, protocol, security, and differentiation documentation with the implementation.

### Honest status & constraints

- **Current Stellar network protocol**: Protocol 29 on Mainnet and Testnet.
- **QuorumScope verified maximum**: Protocol 29 for the CAP-77 workflow.
- **Protocol 30**: Upcoming and not active; no Protocol 30 support claim is made.
- **Live Testnet**: Testnet currently has no non-empty freeze state observed by QuorumScope.
- **Deterministic demo**: The non-empty freeze demonstration is fixture data, explicitly not a live Testnet incident.
- **Render free tier**: The staging service may sleep during inactivity; at review time the Render service was also returning `Service Suspended`, so the deployed API requires a manual resume before use.
- **Production readiness**: No production SLA is provided.
- **Security audit**: No formal third-party security audit has been completed; current dependency advisories remain documented in `SECURITY.md`.

## Release v0.1.0-staging (2026-10-07)

QuorumScope Engine `v0.1.0-staging` marks the initial staging deployment of the Stellar Quorum Freeze indexing and preflight service.

### Summary of Deliverables

- **CAP-77 RPC Ingestion**: Ingests `FrozenLedgerKeys` and `FreezeBypassTxs` config entries from Stellar RPC nodes.
- **Persistent Storage**: PostgreSQL schema managing networks, checkpoints, freeze history, bypass transactions, and incident episodes.
- **Axum API Server**: Read endpoints for network status, keys, bypasses, episodes, and impact, plus in-memory preflight simulation.
- **CLI Commands**: `init`, `serve`, `index once`, `index watch`, and `report`.
- **OpenAPI 3.1.0 Contract**: Published at `GET /openapi.json`.
- **Typst Incident Reports**: Automatic compilation of PDF incident summaries.

### Honest Status & Constraints

- **Staging URL**: [https://quorumscope-engine-api.onrender.com](https://quorumscope-engine-api.onrender.com)
- **Target Network**: Stellar Testnet (`https://soroban-testnet.stellar.org`)
- **Protocol 29 Verified Max**: `VERIFIED_PROTOCOL_MAX` is set to 29 for the current CAP-77 compatibility surface. A future network protocol above the maximum will still be reported as `unverified_protocol`.
- **Empty Freeze Set**: Testnet currently holds 0 frozen keys and 0 bypasses. Non-empty freeze set behavior is verified via mocked RPC tests and fixtures.
- **Render Free Tier**: The hosting service spins down during idle periods, causing cold-start latency on initial requests.
- **Production Readiness**: This release is intended for staging evaluation, integration testing, and developer feedback. It is not an enterprise SLA production service.
