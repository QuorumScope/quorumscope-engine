# Testing Guide

QuorumScope Engine employs a multi-tiered test strategy to verify correctness across domain logic, protocol encoding, indexer synchronization, API handlers, and report generation.

## Test Commands

```bash
# Run all workspace unit and integration tests
cargo test --workspace --all-features

# Run formatting check
cargo fmt --all -- --check

# Run clippy with warnings denied
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build documentation with warnings denied
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

## Test Tiers

### 1. Unit & Domain Tests
Located in `crates/*/src/**`:
- `quorumscope-domain`: Validates entity identifiers, network passphrase hashing, and preflight status state machines.
- `quorumscope-xdr`: Tests XDR decoding, base64 handling, account key parsing, and envelope extraction.
- `quorumscope-rpc`: Tests request serialization and response parsing for `getLatestLedger` and `getLedgerEntries`.
- `quorumscope-freeze`: Tests state-machine transitions: key addition, unfreezing, and bypass hash sets.
- `quorumscope-preflight`: 19 comprehensive unit tests covering candidate transactions across source accounts, destination accounts, trustlines, Soroban footprints, bypass hashes, offers, and unsupported operations.

### 2. Mocked RPC Tests
Located in `crates/quorumscope-indexer/tests/sync_mocked.rs`:
- Uses `wiremock` to simulate Stellar RPC node endpoints.
- Verifies that the indexer correctly ingests non-empty freeze sets and active bypass transactions without requiring a live network.
- Tests unfreeze transitions, incident episode opening and closing, and reconciliation behavior.

### 3. Database Contract Tests
Located in `crates/quorumscope-storage/tests` and `crates/quorumscope-api/tests`:
- Require a real PostgreSQL instance specified by `API_TEST_DATABASE_URL`.
- Verify schema migrations, foreign key constraints, and transactional consistency.
- Validate that all implemented API routes conform to the OpenAPI 3.0 specification.

### 4. Report Generation Tests
Located in `crates/quorumscope-report/src/lib.rs`:
- Verifies that the Typst template compiles successfully and outputs a valid non-empty PDF document for simulated incident episodes.
