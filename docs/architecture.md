# Architecture

QuorumScope Engine is composed of modular components designed for high-fidelity Stellar Quorum Freeze indexing, preflight analysis, and data delivery.

## High-Level Topology

```
                      +-----------------------------+
                      |   Stellar RPC Node          |
                      |   (Testnet Soroban RPC)     |
                      +--------------+--------------+
                                     |
                                     | JSON-RPC (getLedgerEntries, getLatestLedger)
                                     v
                      +-----------------------------+
                      |   QuorumScope Indexer       |
                      |   - Snapshot comparator     |
                      |   - Episode detector        |
                      |   - Evidence recorder       |
                      +--------------+--------------+
                                     |
                                     | SQL transactions (sqlx)
                                     v
                      +-----------------------------+
                      |   PostgreSQL Database       |
                      |   - Networks & Checkpoints  |
                      |   - Freeze changes & keys   |
                      |   - Bypass transactions     |
                      |   - Episodes & Evidence     |
                      +--------------+--------------+
                                     |
                                     | Connection Pool (PgPool)
                                     v
+-----------------------------+      |
|   Typst Report Generator    | <----+
|   - quorumscope report      |      |
+-----------------------------+      v
                      +-----------------------------+
                      |   Axum API Server           |
                      |   - REST Read Endpoints     |
                      |   - Preflight Engine        |
                      |   - OpenAPI 3.0 Specs       |
                      +--------------+--------------+
                                     |
                                     | HTTPS / JSON
                                     v
                      +-----------------------------+
                      |   QuorumScope Console & SDK |
                      +-----------------------------+
```

## Component Breakdown

### 1. Stellar RPC Client (`quorumscope-rpc`)
- Connects to standard Soroban RPC nodes via HTTP POST.
- Executes `getLatestLedger` to determine the latest closed ledger sequence number and active protocol version.
- Executes `getLedgerEntries` for keys with `ConfigSettingId::ConfigSettingFrozenLedgerKeys` and `ConfigSettingId::ConfigSettingFreezeBypassTxs`.
- Handles RPC response timeouts, JSON-RPC envelopes, and serialization errors.

### 2. Ingestion & Synchronization Pipeline (`quorumscope-indexer`)
- **One-Shot Mode (`index once`)**: Queries the RPC node, decodes current entries, verifies stored state, applies delta changes, advances checkpoint markers, and exits with code 0.
- **Watch Mode (`index watch`)**: Executes a persistent polling loop with a configurable interval (`POLLING_INTERVAL_SEC`). Compares each observed snapshot against the previous state:
  - If a key hash is newly present, a freeze event is recorded.
  - If a previously frozen key hash is absent, an unfreeze event is recorded.
  - If any key remains frozen, a freeze episode is kept open.
  - When all keys unfreeze, the active episode is closed.
- **Reconciliation**: On every poll cycle, the indexer verifies that the set of keys in `current_frozen_keys` matches the RPC response exactly. Any discrepancies trigger automatic reconciliation repair.

### 3. Preflight Simulation Engine (`quorumscope-preflight`)
- Accepts a candidate transaction envelope encoded in base64.
- Decodes the envelope using `stellar-xdr` and extracts all accessed ledger keys across:
  - Transaction source account.
  - Operation source accounts.
  - Destination accounts and trustlines.
  - Claimable balance IDs and Liquidity Pool IDs.
  - Soroban contract invocations and data footprint entries (read-only and read-write keys).
- Computes candidate transaction hashes and checks the active bypass list.
- Evaluates the transaction against the current frozen key set:
  - Returns `clear` if no frozen entries are touched.
  - Returns `blocked_validation` if an account, trustline, or footprint key is frozen without a bypass.
  - Returns `allowed_by_bypass` if a matching bypass transaction hash is active.
  - Returns `dex_conditional` if the transaction involves decentralized exchange operations (offers or path payments) that could be affected by liquidity conditions.

### 4. Storage Subsystem (`quorumscope-storage`)
- Backed by PostgreSQL 16.
- Enforces strict foreign key integrity between networks, checkpoints, episodes, and evidence records.
- Stores historical state changes immutably with ledger sequence numbers and timestamps.

### 5. API Server (`quorumscope-api`)
- Built with Axum, Tokio, and Tower.
- Exposes read endpoints for networks, status, freeze states, keys, episodes, bypasses, and impact.
- Generates OpenAPI 3.0 schema directly from code annotations via utoipa.
- Implements security middleware: unique request IDs (`x-request-id`), sanitized error responses, Prometheus metrics, and CORS validation.
