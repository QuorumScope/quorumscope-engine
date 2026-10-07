# Security Policy

## Scope

QuorumScope Engine is the backend indexing and analysis service for Stellar Quorum Freeze (CAP-77). Its primary responsibilities are reading public ledger entries from Stellar RPC nodes, persisting freeze state changes to PostgreSQL, providing transaction preflight simulation, and exposing read APIs.

This policy applies to:
- The QuorumScope Engine codebase in `QuorumScope/quorumscope-engine`.
- Public API endpoints provided by the Axum server (`serve` subcommand).
- Command-line interface utilities (`init`, `index`, `serve`, `report`).
- Database migration and state storage routines.

## Supported versions

| Version | Status | Supported |
| --- | --- | --- |
| `v0.1.x` | Staging / Active Development | Yes |
| Earlier commits | Untagged | No |

Security patches will be released against the active development line and tagged releases.

## Private key and custody policy

- **No Private Keys**: QuorumScope Engine does not manage, generate, accept, or store secret keys or seed phrases.
- **No Custody**: The engine does not hold digital assets, sign transactions, or administer Stellar accounts.
- **No Transaction Submission**: The engine never submits transactions to the Stellar network. Candidate transactions submitted to the preflight endpoint are evaluated entirely in memory against observed freeze sets and are discarded after analysis.

## Transaction handling and privacy

The `POST /api/v1/preflight` endpoint accepts candidate transaction envelopes encoded as base64 XDR:
- **No Persistence**: Transaction XDR is never written to disk, saved to the database, or retained in persistent caches.
- **No Logging**: Request logs record only standard HTTP metadata: timestamp, HTTP method, path, response status code, and execution latency. Transaction bodies and decoded operations are explicitly omitted from logs.
- **Size Bounds**: Payloads sent to `/api/v1/preflight` are enforced by a strict body limit of 256 KiB (`DefaultBodyLimit::max(262_144)`). Requests exceeding this limit receive an immediate `413 Payload Too Large` error.

## Database and secrets policy

- The engine requires database credentials solely for communicating with its PostgreSQL storage layer (`DATABASE_URL`).
- All queries utilize parameterized statements via `sqlx` to protect against SQL injection vulnerabilities.
- Internal database error messages and connection exceptions are logged server-side with structured error contexts and are never leaked to external clients. Client responses return standardized error envelopes with random `request_id` values for operational tracing.

## CORS policy

Cross-Origin Resource Sharing (CORS) is strictly opt-in:
- When the `--allowed-origins` command-line flag or `ALLOWED_ORIGINS` environment variable is unset, no CORS headers are returned, preventing unauthorized web origins from querying the API.
- In staging and production deployments, origins are restricted explicitly to authorized console hosts (such as `https://quorumscope-console.vercel.app`).

## Incident report generation and privacy

The `quorumscope report` command compiles PDF incident summaries using Typst:
- Reports contain only cryptographic evidence hashes, affected public ledger keys, ledger sequence numbers, and protocol state transitions.
- No user identity, IP address, or external metadata is recorded in generated reports.

## Dependency expectations

- Rust dependencies are tracked in `Cargo.lock` and audited regularly using `cargo deny` and `cargo audit`.
- Releases target the pinned Rust version specified in `rust-toolchain.toml`.
- Stellar protocol types are decoded using the official `stellar-xdr` crate.

## Audit status

QuorumScope Engine is an open-source research and monitoring tool developed for Stellar Quorum Freeze visibility. It has **not** undergone a formal third-party security audit. Operators deploying this software are responsible for evaluating its fitness for their security posture.

## Vulnerability reporting

If you discover a security vulnerability in QuorumScope Engine, please report it privately through GitHub:
1. Navigate to the repository: [QuorumScope/quorumscope-engine](https://github.com/QuorumScope/quorumscope-engine).
2. Go to the **Security** tab and click **Report a vulnerability**.
3. Provide a detailed summary including:
   - Affected crate, CLI command, or API route.
   - Step-by-step reproduction instructions or proof-of-concept.
   - Observed impact and potential attack vectors.
4. Do not include private credentials or sensitive production data in the report.

Do not open public GitHub issues or discussions for undisclosed security vulnerabilities. We will review reports promptly and coordinate a patch and disclosure timeline.
