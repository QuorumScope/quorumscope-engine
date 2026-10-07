<div align="center">

<img src="assets/quorumscope-engine-banner.png" alt="QuorumScope Engine Banner" width="100%" />

# QuorumScope Engine

QuorumScope engine for Stellar Quorum Freeze indexing, preflight analysis, impact data, API, CLI, and reports.

[![Rust CI](https://github.com/QuorumScope/quorumscope-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/QuorumScope/quorumscope-engine/actions/workflows/rust.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Staging API](https://img.shields.io/badge/Staging_API-Online-brightgreen.svg)](https://quorumscope-engine-api.onrender.com)
[![OpenAPI](https://img.shields.io/badge/OpenAPI-3.0.3-orange.svg)](https://quorumscope-engine-api.onrender.com/openapi.json)
[![Protocol Max](https://img.shields.io/badge/Protocol_Verified_Max-28-blue.svg)](docs/versions.md)
[![Testnet Connected](https://img.shields.io/badge/Testnet-Connected-success.svg)](https://quorumscope-engine-api.onrender.com/api/v1/network)
[![Branch Protected](https://img.shields.io/badge/Branch_Protection-Active-success.svg)](https://github.com/QuorumScope/quorumscope-engine/tree/main)

[Documentation](https://quorumscope.github.io/quorumscope-engine/) · [Staging API](https://quorumscope-engine-api.onrender.com) · [OpenAPI JSON](https://quorumscope-engine-api.onrender.com/openapi.json) · [Latest Release](https://github.com/QuorumScope/quorumscope-engine/releases/latest) · [Security](SECURITY.md) · [Contributing](CONTRIBUTING.md)

</div>

---

## What is QuorumScope Engine?

QuorumScope Engine is the backend protocol-processing layer for Stellar Quorum Freeze (CAP-77). It connects to a Stellar RPC node, tracks active freeze states and bypass lists, stores state changes with cryptographic evidence in PostgreSQL, and serves the data through an HTTP API and a command-line interface.

The engine also provides preflight simulation to test whether candidate transactions touch frozen keys before submission, calculates blast radius impact data, and compiles freeze episodes into PDF incident reports.

## Why it exists

CAP-77 introduces protocol-level freeze mechanisms to Stellar. When an account, trustline, or smart contract storage key is frozen by quorum action, transactions touching those ledger entries fail validation unless explicitly included in a freeze bypass list.

Developers and operators need visibility into:
- Which ledger keys are actively frozen and when they were modified.
- Whether a planned transaction will be blocked by active freeze conditions.
- How fresh the observed freeze state is compared to the network ledger tip.
- What accounts and contracts are affected by an active freeze episode.

QuorumScope Engine turns low-level RPC state into queryable API endpoints, deterministic preflight checks, and verifiable incident records.

## Current staging status

| Service | Host | Status |
| --- | --- | --- |
| HTTP API | Render | [Staging API](https://quorumscope-engine-api.onrender.com) |
| OpenAPI Specification | Render | [OpenAPI JSON](https://quorumscope-engine-api.onrender.com/openapi.json) |
| Database | Supabase | Managed PostgreSQL 16 |
| Target Network | Stellar Testnet | RPC at `https://soroban-testnet.stellar.org` |

Staging notes:
- The staging API runs on Render free tier infrastructure. It spins down during inactivity, so cold starts can take up to 60 seconds. A waking instance may temporarily report stale freshness until the next poll cycle completes.
- The engine enforces `VERIFIED_PROTOCOL_MAX=28`. Stellar Testnet currently reports Protocol 29, so the API reports `unverified_protocol` compatibility. This is expected behavior.
- Stellar Testnet currently contains an empty freeze set (0 frozen keys and 0 bypasses).
- Non-empty freeze set handling and unfreeze transitions are verified using integration tests, wiremock RPC tests, and fixtures rather than live testnet data.

## Features

- **Stellar RPC Ingestion**: Polls `getLedgerEntries` for `FrozenLedgerKeys` and `FreezeBypassTxs` config entries and queries `getLatestLedger`.
- **Freeze State Snapshots**: Tracks active frozen keys, bypass transaction hashes, and sequence checkpoints.
- **Freeze and Unfreeze History**: Records key changes with timestamps, ledger sequences, and transaction context.
- **Evidence References**: Links every state transition to cryptographic evidence hashes and observation logs.
- **Preflight Analysis**: Analyzes base64 transaction envelopes against active freeze sets across accounts, trustlines, claimable balances, and Soroban contract storage.
- **Impact API**: Classifies affected resources into direct and protocol-derived evidence categories.
- **Freshness and Compatibility Tracking**: Reports source ledger sequence, lag behind network tip, observation age, and protocol verification status.
- **OpenAPI 3.0 Specification**: Exposes route specifications at `/openapi.json`.
- **Command-Line Interface**: Provides `init`, `serve`, `index once`, `index watch`, and `report` subcommands.
- **Incident PDF Reports**: Generates formal incident summaries using Typst.
- **Security Controls**: Implements request ID tracking (`x-request-id`), sanitized error responses, configurable CORS, and strict body limits.

## Architecture

```
+---------------------------+
|    Stellar RPC Node       |
| (Testnet / Soroban RPC)   |
+-------------+-------------+
              |
              | getLedgerEntries / getLatestLedger
              v
+---------------------------+       +---------------------------+
|    QuorumScope Indexer    | ----> |   PostgreSQL Database     |
| (index once / watch mode) |       | (Snapshots, Keys, Events) |
+---------------------------+       +-------------+-------------+
                                                  |
                                                  v
+---------------------------+       +---------------------------+
|    QuorumScope Console    | <---- |     Axum HTTP API         |
| (Next.js Web / SDK)       |       | (serve / preflight / read)|
+---------------------------+       +-------------+-------------+
                                                  |
                                                  v
                                    +---------------------------+
                                    |     Typst PDF Reports     |
                                    | (quorumscope report)      |
                                    +---------------------------+
```

## API overview

All endpoints return JSON and include an `x-request-id` header.

| Method | Endpoint | Description |
| --- | --- | --- |
| `GET` | `/health/live` | Process liveness probe |
| `GET` | `/health/ready` | Readiness probe confirming database connectivity |
| `GET` | `/api/v1/network` | Network identification, ledger checkpoints, and freshness |
| `GET` | `/api/v1/status` | Indexer synchronization status, ingestion lag, and compatibility |
| `GET` | `/api/v1/freeze-state` | Active freeze set, bypass entries, and checkpoint metadata |
| `GET` | `/api/v1/frozen-keys` | Paginated list of frozen keys with kind and active filters |
| `GET` | `/api/v1/frozen-keys/{id}` | Detailed key information, decoded structure, and change history |
| `GET` | `/api/v1/bypasses` | Paginated list of active freeze bypass transaction hashes |
| `GET` | `/api/v1/incidents` | Recorded freeze episodes with opened and closed ledgers |
| `GET` | `/api/v1/incidents/{id}` | Details for a specific freeze episode |
| `GET` | `/api/v1/incidents/{id}/timeline` | Sequential evidence timeline for a freeze episode |
| `POST` | `/api/v1/preflight` | Evaluates a candidate transaction against the current freeze state |
| `GET` | `/api/v1/impact` | Paginated impact records filtered by evidence class and key kind |
| `GET` | `/openapi.json` | OpenAPI 3.0 specification |
| `GET` | `/metrics` | Prometheus operational metrics |

## Quick start

### Prerequisites

- Rust 1.98.1 or higher (see `rust-toolchain.toml`)
- PostgreSQL 16
- Typst (optional, required only for `report` generation)

### Local setup

1. Clone the repository and build the workspace:

```bash
git clone https://github.com/QuorumScope/quorumscope-engine.git
cd quorumscope-engine
cargo build
```

2. Start PostgreSQL:

```bash
make db-up
```

3. Set connection variables:

```bash
export DATABASE_URL="postgres://quorumscope:quorumscope_dev@localhost:5432/quorumscope"
export STELLAR_RPC_URL="https://soroban-testnet.stellar.org"
```

4. Apply migrations:

```bash
cargo run -p quorumscope-cli -- init
```

5. Run a one-shot index pass against testnet:

```bash
cargo run -p quorumscope-cli -- index once
```

6. Start continuous indexing in the background or a separate terminal:

```bash
cargo run -p quorumscope-cli -- index watch
```

7. Start the API server:

```bash
cargo run -p quorumscope-cli -- serve --bind 127.0.0.1:8080
```

Verify the server:

```bash
curl http://127.0.0.1:8080/health/ready
curl http://127.0.0.1:8080/api/v1/network
```

## Configuration

| Variable | CLI Flag | Default | Description |
| --- | --- | --- | --- |
| `DATABASE_URL` | `--db-url` | `postgres://localhost/quorumscope` | PostgreSQL connection URL |
| `STELLAR_RPC_URL` | `--rpc-url` | `https://soroban-testnet.stellar.org` | Stellar RPC endpoint |
| `NETWORK_NAME` | `--network-name` | `testnet` | Network identifier in storage |
| `NETWORK_PASSPHRASE` | `--network-passphrase` | `Test SDF Network ; September 2015` | Stellar network passphrase |
| `API_BIND` | `--bind` | `127.0.0.1:8080` | Bind address and port for HTTP server |
| `VERIFIED_PROTOCOL_MAX` | `--verified-protocol-max` | `28` | Highest protocol version verified by this release |
| `STALE_AFTER_SEC` | `--stale-after-sec` | `300` | Seconds before inactivity reports state as stale |
| `MAX_LAG_LEDGERS` | `--max-lag-ledgers` | `10` | Allowable ledger lag before marking state behind |
| `ALLOWED_ORIGINS` | `--allowed-origins` | None | Comma-separated allowed CORS origins |
| `POLLING_INTERVAL_SEC` | `--polling-interval` | `5` | Poll interval in seconds for indexer watch mode |
| `LOG_LEVEL` | `--log-level` | `info` | Logging verbosity |

## Verification

The test suite enforces verification boundaries across testing categories:

| Category | Scope | Verification Evidence |
| --- | --- | --- |
| **Live Verified** | Real Stellar testnet RPC, PostgreSQL 16 | Migration application, empty freeze set indexing, checkpoint advancement, reconciliation repair, live preflight evaluation, OpenAPI matching. |
| **Integration Verified** | Real PostgreSQL database, synthetic data | Full episode lifecycles (open/close), preflight status codes, impact records and filters, error envelopes with request IDs, CORS enforcement. |
| **Fixture Verified** | Offline fixtures | 19 preflight operation scenarios, freeze and bypass state-machine transitions, Typst PDF generation from synthetic episodes. |
| **Mocked** | Wiremock RPC server, real PostgreSQL | Decoding non-empty freeze sets and bypass lists, episode generation, evidence recording without live network dependency. |
| **Unverified** | Protocol 29 and live non-empty state | Live network behavior with an active non-empty freeze set has not occurred on testnet. Protocol 29 compatibility is unverified. |

## Limitations

- **No Live Non-Empty Freeze Set**: Stellar Testnet currently contains 0 frozen keys. Live RPC decoding of non-empty freeze sets has only been verified using mocked RPC responses and local fixtures.
- **Protocol 29 Status**: Stellar Testnet operates on Protocol 29. Because the engine is verified up to Protocol 28 (`VERIFIED_PROTOCOL_MAX=28`), the system reports `unverified_protocol`.
- **Infrastructure Hibernation**: The staging API on Render free tier sleeps during inactivity, resulting in cold starts up to 60 seconds and temporary stale freshness status until refreshed.
- **Uncollected Impact Classes**: Impact classes requiring transaction history (`recently_observed`, `dependency_observed`, `inferred`) are not collected because transaction history indexing is not implemented.
- **Ledger Close Times**: Ledger close timestamps are not indexed or stored in the database.
- **Audit Status**: The codebase has not undergone a formal third-party security audit.

## Security

- **No Secret Keys**: The engine requires no secret keys, signer seeds, or account credentials.
- **Non-Custodial**: The engine does not hold assets, manage custody, or operate signing infrastructure.
- **No Transaction Submission**: The engine never submits transactions to the Stellar network. Preflight evaluates candidate transactions locally in memory.
- **Transaction Privacy**: Transaction envelopes submitted to `POST /api/v1/preflight` are never persisted to disk, written to the database, or included in server logs. Request logging records only method, path, status, and latency.
- **Input Bounds**: Preflight request payloads are restricted to a maximum of 256 KiB.
- **Database Sanitization**: Internal database error details are logged server-side and never returned in client error responses.

For reporting vulnerabilities, see [SECURITY.md](SECURITY.md).

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on workspace setup, coding conventions, testing requirements, and commit guidelines.

Key standards:
- All changes must pass `cargo fmt`, `cargo clippy`, and `cargo test`.
- Use conventional commit messages.
- Commit one logical change per commit.
- Never use `git add .` to stage files.

## Roadmap

- Protocol 29 verification and compatibility testing.
- Live non-empty freeze set verification when protocol events occur on public networks.
- Always-on indexer deployment on dedicated paid hosting.
- Transaction history indexing to support `recently_observed` and `dependency_observed` impact classes.
- Storage and indexing of ledger close timestamps.
- Documentation site enhancements and automated API explorer integration.
- Release hardening and packaging.

For detailed release history and milestone planning, see [CHANGELOG.md](CHANGELOG.md) and [ROADMAP.md](ROADMAP.md).
