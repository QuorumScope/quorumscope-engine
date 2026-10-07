# Contributing to QuorumScope Engine

Thank you for contributing to QuorumScope Engine. This document outlines development standards, testing workflows, and policies for contributing to the protocol processing layer.

## Development setup

### Prerequisites

- **Rust**: 1.98.1 or higher (see `rust-toolchain.toml`). Install via [rustup](https://rustup.rs).
- **PostgreSQL**: Version 16. Install locally or use Docker via `make db-up`.
- **Typst**: Version 0.11.0 or higher (required only if generating PDF incident reports via `quorumscope report`).

### Local setup

1. Clone the repository:

```bash
git clone https://github.com/QuorumScope/quorumscope-engine.git
cd quorumscope-engine
```

2. Build the workspace:

```bash
cargo build
```

3. Start PostgreSQL and apply migrations:

```bash
make db-up
export DATABASE_URL="postgres://quorumscope:quorumscope_dev@localhost:5432/quorumscope"
cargo run -p quorumscope-cli -- init
```

4. Verify formatting, linting, and tests:

```bash
make check
```

## Workspace structure

QuorumScope Engine is organized as a Cargo workspace with targeted crates:

| Crate | Path | Responsibility |
| --- | --- | --- |
| `quorumscope-domain` | `crates/quorumscope-domain` | Core domain types, state models, and validation logic |
| `quorumscope-xdr` | `crates/quorumscope-xdr` | Stellar XDR codecs, envelope decoders, and entry keys |
| `quorumscope-rpc` | `crates/quorumscope-rpc` | Stellar RPC client for ledger queries and status checks |
| `quorumscope-freeze` | `crates/quorumscope-freeze` | Freeze state-machine, bypass reconciliation, and change tracking |
| `quorumscope-preflight` | `crates/quorumscope-preflight` | Preflight simulation of candidate transactions against freeze sets |
| `quorumscope-storage` | `crates/quorumscope-storage` | PostgreSQL database persistence, schema migrations, and repositories |
| `quorumscope-indexer` | `crates/quorumscope-indexer` | Ingestion pipeline, one-shot indexing, and watch loop |
| `quorumscope-api` | `crates/quorumscope-api` | Axum HTTP server, OpenAPI metadata, metrics, and middleware |
| `quorumscope-report` | `crates/quorumscope-report` | Incident report compilation to PDF using Typst |
| `quorumscope-cli` | `crates/quorumscope-cli` | Command-line interface entry points and configuration binding |

## Exact test commands

Run the standard check suite before submitting changes:

```bash
# Check code formatting
cargo fmt --all -- --check

# Run linter with all warnings denied
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all unit, fixture, and mock tests
cargo test --workspace --all-features

# Build workspace documentation with warnings denied
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

### PostgreSQL integration tests

To run database integration tests that verify migration application and episode lifecycles, set `API_TEST_DATABASE_URL`:

```bash
export API_TEST_DATABASE_URL="postgres://quorumscope:quorumscope_dev@localhost:5432/quorumscope_test"
cargo test --test integration_test --all-features
```

## Live testnet checks policy

- Live testnet indexing and RPC verification are reserved for manual smoke testing and dedicated maintenance tasks.
- Automated CI workflows must not depend on external testnet RPC availability.
- All edge cases, including non-empty freeze sets and unfreeze episodes, must be tested via wiremock and offline fixtures.
- When performing live testnet verification, document observed sequence numbers, protocol versions, and freshness states in [docs/verification.md](docs/verification.md).

## No secrets policy

- Never commit private keys, signer seeds, API secrets, or internal database passwords.
- The engine operates entirely in read-only mode against public Stellar RPC endpoints and requires no signer credentials.
- All local development and testing must use dummy credentials or local test databases.

## Coding standards

- Write idiomatic Rust adhering to the latest stable edition.
- Treat compiler warnings and clippy recommendations as errors.
- Ensure public items in domain and API crates are documented with doc comments.
- Keep error messages descriptive, structured, and free of sensitive internal state.
- Do not use em dashes in documentation or error text.

## Updating OpenAPI specifications

When adding or modifying API routes:
1. Update route definitions and utoipa annotations in `crates/quorumscope-api`.
2. Run the API test suite to verify route coverage:
   ```bash
   cargo test -p quorumscope-api --test engine_contract
   ```
3. Export the updated OpenAPI specification to `openapi/openapi.json`.
4. Ensure the schema accurately reflects request parameters, responses, and error envelopes.

## Updating documentation

- Documentation files live in the `docs/` directory.
- When architecture, API endpoints, or verification boundaries change, update the corresponding markdown documents in `docs/` and summarize the change in `README.md`.
- Keep documentation factual. Never invent metrics, performance claims, or production endorsements.

## Commits and pull requests

### Commit hygiene

- Follow Conventional Commits format:
  - `feat: add claimable balance preflight support`
  - `fix: correct lag ledger calculation in status handler`
  - `docs: update verification boundaries for protocol 29`
  - `test: add wiremock test for multiple frozen trustlines`
- Group changes into logical, cohesive units.
- **Never use `git add .`** Stage specific files explicitly.
- Do not add automated co-author trailers.

### Pull request checklist

Before opening a pull request, verify:
- [ ] `cargo fmt --all -- --check` passes with no differences.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` completes without warnings.
- [ ] `cargo test --workspace --all-features` passes all unit and integration tests.
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` completes cleanly.
- [ ] Documentation and OpenAPI specifications are updated if routes or types changed.
- [ ] Commits are logical and clearly titled.

### Issue guidance

- Use the GitHub issue templates for bug reports and feature proposals.
- Clearly describe reproducible steps, expected versus observed results, and environment details.
- For security-sensitive findings, do not file public issues; consult [SECURITY.md](SECURITY.md).
