# QuorumScope Engine

QuorumScope reads Stellar Quorum Freeze (CAP-77) state from a Stellar RPC node, stores it in PostgreSQL, and serves it over an HTTP API. The API includes transaction preflight, which compares a transaction envelope with the active freeze set.

## What is implemented

- Indexer: reads `FrozenLedgerKeys` and `FreezeBypassTxs` through `getLedgerEntries`, records freeze and bypass changes with evidence, derives freeze episodes, and checks stored state against the network on every poll.
- API: network, freeze state, frozen keys with history, bypasses, episodes and timelines, impact, status, preflight, and health endpoints. The contract is [openapi/openapi.json](openapi/openapi.json).
- Reports: `quorumscope report` writes a PDF for one freeze episode.

See [docs/api.md](docs/api.md), [docs/preflight.md](docs/preflight.md), and [docs/verification.md](docs/verification.md) for behavior and for what has been checked against a live network.

## Requirements

- Rust 1.98.1 (see `rust-toolchain.toml`)
- PostgreSQL 16
- Typst, only for `quorumscope report`

## Local setup

```bash
export POSTGRES_PASSWORD=choose-a-password
make db-up
export DATABASE_URL=postgres://quorumscope:$POSTGRES_PASSWORD@localhost:5432/quorumscope
cargo run -p quorumscope-cli -- init
cargo run -p quorumscope-cli -- index once
cargo run -p quorumscope-cli -- serve
```

`.env.example` lists every variable the binary reads. Run `quorumscope --help` and `quorumscope <command> --help` for flags.

## Commands

| Command | Purpose |
| --- | --- |
| `init` | Apply migrations |
| `index once` | Read the network once and exit |
| `index watch` | Poll until SIGINT or SIGTERM |
| `serve` | Serve the HTTP API and apply migrations at start |
| `report --incident <uuid> --output <file>` | Write an episode PDF |

State is stored per network name (`NETWORK_NAME`, default `testnet`). The network passphrase is stored with it and cannot change for an existing name.

## Tests

```bash
make check
```

Set `API_TEST_DATABASE_URL` to a PostgreSQL URL to run the database contract tests. They are skipped when it is unset. CI sets it.
