# Verification status

Each item below says how it was checked. "Live" means a real Stellar RPC node and a real PostgreSQL database. The live runs used Stellar testnet through `https://soroban-testnet.stellar.org` on 2026-10-04 (protocol version 29 at that time).

## Protocol version

`VERIFIED_PROTOCOL_MAX` defaults to 28. It means the engine was checked against the Protocol 28 XDR and tooling line and current live Soroban Testnet RPC responses. It does not mean every CAP-77 edge case is proven on a live network. The CAP-77 rules were reviewed against the specification text on 2026-10-05, with open points listed in [preflight.md](preflight.md). A network reporting a higher protocol, such as the 29 testnet reported on 2026-10-04, is shown as `unverified_protocol`.

## Live verified

- Migrations apply to a fresh PostgreSQL 16 database and create the schema.
- `index once` reads `FrozenLedgerKeys` and `FreezeBypassTxs`, registers the network, stores both configuration snapshots, advances the checkpoint, records the network ledger and protocol version, and reconciles.
- `index watch` polled repeatedly, advanced the checkpoint each poll, kept its checkpoint across a restart, and exited with status 0 on SIGTERM and on SIGINT. Repeated polls added no snapshots and no change records.
- Reconciliation repair: a key inserted directly into `current_frozen_keys` was removed by the next poll, recorded as an unfreeze, and the poll reconciled.
- `serve` against that database returned valid JSON from every endpoint that does not need an id: network, freeze-state, frozen-keys, bypasses, incidents, impact, status, health. The served `/openapi.json` equals the committed `openapi/openapi.json`.
- `POST /api/v1/preflight` with a synthetic transaction returned `clear` against the live freeze state, and `invalid_input` for bad XDR.

Testnet had an empty freeze set during these runs. Nothing live shows the engine handling a non-empty freeze set.

A fresh clone of `origin/main` at `1f32203` passed format, clippy with warnings denied, all workspace tests with a PostgreSQL database, and `cargo doc` with warnings denied.

## Integration verified (real PostgreSQL, constructed data)

Run in CI and locally with `API_TEST_DATABASE_URL`:

- A poll that freezes a key opens an episode, records events, evidence references, and an impact snapshot. Repeating the poll adds nothing. Unfreezing the last key closes the episode.
- Preflight `clear`, `blocked_validation` (naming the key and path), `allowed_by_bypass`, `invalid_input`, and `state_unavailable` for unindexed and stale networks.
- Impact records, filters, and validation errors. Key detail with decoded content and history. Freshness and compatibility fields.
- Error envelope with `request_id` in the body and header. `413` for oversized preflight bodies. CORS only for configured origins.
- Every implemented route is present in the OpenAPI document.

## Fixture verified

- Preflight analysis (19 unit tests) on synthetic transaction envelopes: account, trustline, destination, source, Soroban footprint, bypass hash, DEX, path payment, unsupported operations.
- Freeze and bypass change computation.
- Report rendering through Typst 0.11.0 to a non-empty PDF, and `quorumscope report` on a synthetic episode row. No real episode has been reported because testnet had none.
- A `getLatestLedger` response parse.

## Mocked

- `crates/quorumscope-indexer/tests/sync_mocked.rs` simulates the RPC server with wiremock and uses real PostgreSQL. It feeds the indexer a non-empty freeze set and bypass list with synthetic keys, then an empty set. It checks stored keys, bypasses, the opened and resolved episode, evidence references, no duplicate history on a repeated poll, and reconciliation. This is the only coverage of the indexer decoding a non-empty set, so it shows the code path works, not that a live network encodes it the same way.

## Not verified

- Behavior when the live network has frozen keys or bypasses. The preflight rules were reviewed against the CAP-77 text on 2026-10-05, not against a running network. Open points are listed in [preflight.md](preflight.md).
- Whether the bypass content hash includes the network ID the way this engine computes it. Fee bump handling is confirmed by the CAP text, but not on a live network.
- The `recently_observed`, `dependency_observed`, and `inferred` impact classes. They need transaction history, which is not indexed.
- Ledger close times. They are not stored.
- A container image. There is no Dockerfile.
- Any deployed API.
