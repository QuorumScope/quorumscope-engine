# Roadmap

This roadmap documents planned improvements and engineering milestones for QuorumScope Engine. Items are prioritized based on protocol developments and operational requirements.

## Current focus: Staging stabilization

- [x] Functional CAP-77 indexing pipeline for `FrozenLedgerKeys` and `FreezeBypassTxs`.
- [x] In-memory preflight simulation covering core Stellar operations and Soroban footprints.
- [x] PostgreSQL persistence with schema migrations.
- [x] OpenAPI 3.0 specification publication.
- [x] Staging deployment on Render backed by Supabase PostgreSQL.
- [x] Wiremock RPC tests and fixture suites covering non-empty freeze sets.

## Near-term milestones

### 1. Protocol 29 verification
- Track Stellar core Protocol 29 stabilization and tooling releases.
- Verify XDR changes and RPC response schemas under Protocol 29.
- Advance `VERIFIED_PROTOCOL_MAX` to 29 once specification compatibility is verified.

### 2. Live non-empty freeze set verification
- Monitor Stellar Testnet for protocol freeze events.
- Validate live RPC ingestion against non-empty freeze sets and active bypass entries.
- Record live evidence captures to validate wiremock and fixture models.

### 3. Dedicated infrastructure
- Migrate indexer watch process to an always-on worker service on paid infrastructure.
- Eliminate cold-start latency from free-tier hibernation.
- Maintain persistent real-time ledger synchronization.

## Long-term milestones

### 4. Extended impact analysis
- Ingest transaction history blocks to identify accounts that interacted with recently frozen keys.
- Populate `recently_observed`, `dependency_observed`, and `inferred` impact classes.
- Provide dependency graphs between smart contracts and frozen data keys.

### 5. Ledger close time support
- Persist ledger close timestamps alongside ledger sequence checkpoints.
- Support precise time-range queries for freeze episodes and unfreeze durations.

### 6. Documentation and release hardening
- Publish automated OpenAPI client generation workflows.
- Expand interactive API documentation and documentation guides.
- Prepare distribution binaries and container image releases.
