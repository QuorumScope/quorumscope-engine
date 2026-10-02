# QuorumScope Engine

The CAP-77 transaction preflight and impact observation engine.

## Requirements
- Rust 1.70+
- Docker & Docker Compose (for DB)

## Setup
To initialize the stack locally:
```bash
make setup
make db-up
make dev
```

## Testing
Run the test suite:
```bash
make check
```

## Indexer
The engine includes a synchronization indexer to fetch current CAP-77 freeze state from a Soroban RPC node and update local PostgreSQL state.

Run one-shot index:
```bash
cargo run -p quorumscope-cli -- index once
```

Run continuously (watch mode):
```bash
cargo run -p quorumscope-cli -- index watch
```

Configuration via environment variables:
- `DATABASE_URL` (default: postgres://localhost/quorumscope)
- `STELLAR_RPC_URL` (default: https://soroban-testnet.stellar.org)
- `POLLING_INTERVAL_SEC` (default: 5)
- `NETWORK_PASSPHRASE`
- `LOG_LEVEL`
