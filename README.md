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
