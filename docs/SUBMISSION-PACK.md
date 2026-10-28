# QuorumScope Engine submission pack

This document is a short, factual orientation for a reviewer. It summarizes what Engine is, why it
exists, what is verifiably built, and what is not yet proven. It links to the full documentation set
rather than repeating it.

## Project

- Project: QuorumScope
- Repository: [QuorumScope/quorumscope-engine](https://github.com/QuorumScope/quorumscope-engine)
- Role: CAP-77 indexing, protocol interpretation, preflight analysis, impact data, API, CLI, and
  incident reporting
- License: Apache-2.0
- Current staging release: [`v0.1.1-staging`](https://github.com/QuorumScope/quorumscope-engine/releases/tag/v0.1.1-staging)
- Current Stellar protocol: 29
- QuorumScope verified protocol maximum: 29 (Protocol 30 is not supported or claimed; see
  [Protocol Versions](versions.md))
- Sibling repository: [QuorumScope/quorumscope-console](https://github.com/QuorumScope/quorumscope-console)

## One-line description

QuorumScope Engine turns Stellar Quorum Freeze (CAP-77) protocol state into inspectable operational
data: which ledger keys are frozen, how fresh that state is, and whether a candidate transaction
would be blocked or covered by a freeze bypass.

## Problem

CAP-77 lets validators put specific ledger keys into an on-chain freeze set, with an explicit
transaction bypass list. That state is authoritative in the raw RPC and XDR responses, but raw
config entries do not tell an operator or developer, in a usable form:

- which keys are frozen, and since when;
- whether the data they are looking at is current or stale relative to the network tip;
- which transactions carry freeze bypass status;
- whether a transaction they are about to submit touches frozen state;
- what can be said about the impact of an active freeze, with evidence; or
- how freeze state changed over the life of an incident.

## Solution

Engine addresses this by:

- reading CAP-77 configuration and ledger entries from a Stellar RPC node;
- reconciling that state into frozen keys, bypass entries, and freeze episodes;
- persisting observations and state transitions in PostgreSQL with evidence references;
- exposing the result through an HTTP API and OpenAPI 3.1.0 contract;
- supporting a CLI for indexing, serving, and PDF incident reports;
- analyzing a candidate transaction XDR in memory through a preflight endpoint; and
- tracking and reporting data freshness and protocol compatibility alongside every response.

## Why Stellar

QuorumScope is Stellar-specific, not a generic chain monitor with a renamed network selector:

- CAP-77 is Stellar protocol state defined by Stellar's own XDR and RPC config entries.
- The Engine decodes Stellar ledger keys (accounts, trustlines, claimable balances, Soroban
  contract storage) using the pinned `stellar-xdr` crate for the current protocol line.
- Preflight analysis interprets Stellar transaction envelopes and operation semantics, including
  Soroban footprints, to determine whether an operation touches frozen state.
- Freeze and bypass behavior is governed by CAP-77 rules, not a generic "watch an address" model.

## Architecture

Engine is the indexing, storage, and API layer: Stellar RPC node to indexer to PostgreSQL to an
Axum HTTP API, with an optional Typst report generator and the Console as a downstream consumer of
the API. See [Architecture](architecture.md) for the full component and data-flow description; it is
not duplicated here.

## What is verifiably built

- Protocol 29 compatibility verification against the current Stellar Mainnet and Testnet XDR line
  (see [Protocol Versions](versions.md)).
- CAP-77 configuration parsing for frozen-key, frozen-key-delta, bypass, and bypass-delta settings.
- Frozen key reconciliation and freeze/unfreeze history tracking.
- Freeze bypass reconciliation.
- Preflight analysis across accounts, trustlines, claimable balances, and Soroban contract storage.
- Impact analysis with explicit `direct` and `protocol_derived` evidence classes, and explicit
  reporting of evidence classes that are not yet collected.
- PostgreSQL-backed storage with parameterized queries.
- An HTTP API (`/api/v1/...`) and a CLI (`init`, `serve`, `index once`, `index watch`, `report`).
- Typst-based PDF incident reports.
- An OpenAPI 3.1.0 contract at `/openapi.json`, verified to match the Console's SDK snapshot
  byte-for-byte.
- A deterministic non-empty CAP-77 demo covering the full indexing and preflight path (see below).
- CI covering formatting, linting, and PostgreSQL-backed integration tests.
- Current staging release [`v0.1.1-staging`](https://github.com/QuorumScope/quorumscope-engine/releases/tag/v0.1.1-staging).

## Deterministic non-empty proof

### Deterministic fixture verification

`./scripts/demo-nonempty.sh` starts a temporary PostgreSQL 16 container, generates CAP-77 XDR with
the repository's own Stellar XDR types, serves it from a local JSON-RPC fixture, and runs the real
indexer and the real API against it. This exercises the full production chain: fixture RPC, real
Engine indexer, PostgreSQL 16, real Engine API, TypeScript SDK, Console, Freeze Map, and preflight.

Known deterministic results from this path include:

- one frozen account key, with decoded key information;
- one active bypass transaction hash;
- a blocked preflight result (`blocked_validation`) for a transaction touching the frozen key;
- a bypassed preflight result (`allowed_by_bypass`) for a transaction covered by the bypass; and
- non-empty impact and Freeze Map data.

Full detail is in [the deterministic demo](demo.md).

**This is deterministic fixture verification, not a live Testnet freeze.** It proves the Engine's
decoding, persistence, reconciliation, API, and preflight logic handle non-empty state correctly. It
does not claim that a non-empty freeze event has occurred on a public network.

## Live Testnet verification

- Stellar Testnet currently runs Protocol 29, matching QuorumScope's verified maximum of 29.
- The public Testnet freeze set has been empty (0 frozen keys, 0 bypasses) during every observation
  window to date; no non-empty live incident has been observed.
- Previous Engine staging verification succeeded on Render before the current free-tier
  instance-hour allowance was exhausted.
- The staging Engine was previously deployed and verified on Render. The current free-tier
  instance-hour allowance has been exhausted, so the service may presently be suspended. This is an
  infrastructure quota condition, not a substitute for the deterministic verification described
  above.

## Security

- No private keys: the Engine never requires, stores, or processes secret keys or seed phrases.
- No custody: the Engine does not hold assets or sign transactions.
- No transaction submission: candidate transactions sent to preflight are analyzed in memory only
  and are never submitted to the network.
- The preflight endpoint enforces a 256 KiB request body limit.
- Database access uses parameterized queries via `sqlx`.
- CORS origins are explicitly configured; unset configuration disables CORS entirely.
- No formal third-party security audit has been performed.

Rust dependency audit status:

- One remaining known advisory: `RUSTSEC-2023-0071`, in `rsa 0.9.10`, reached through the optional
  `sqlx-mysql` feature path.
- The active QuorumScope configuration uses PostgreSQL, not MySQL.
- No suppression has been added for this advisory.
- `cargo audit` remains non-clean as a result.

See [SECURITY.md](../SECURITY.md) for the disclosure policy. QuorumScope Engine is not described as
free of vulnerabilities.

## Differentiation

QuorumScope is not a wallet, a generic Stellar explorer, a transaction sender, a validator, or a
freeze governance or execution tool. It focuses on inspection: freshness, preflight, impact,
bypass and freeze history, and operational visibility into CAP-77 state that a validator or
governance tool has already produced. See [Differentiation](differentiation.md) for the full
comparison against the one clearly adjacent public project found during review.

## Honest limitations

- No non-empty freeze incident has been observed on public Testnet.
- Transaction-history-backed impact evidence classes (`recently_observed`, `dependency_observed`,
  `inferred`) are not yet collected.
- Ledger close timestamps are not yet persisted; API timestamps reflect engine observation time.
- Protocol 30 is future work and is not claimed as supported.
- The Render free-tier staging deployment is not always-on.
- No formal third-party security audit has been performed.
- One Rust advisory remains through the optional MySQL dependency path (see Security above).

See [Known Limitations](limitations.md) for the complete record.

## Current roadmap

- [#1 Track Protocol 30 readiness after Bell activation](https://github.com/QuorumScope/quorumscope-engine/issues/1)
- [#2 Add transaction-history-backed impact evidence](https://github.com/QuorumScope/quorumscope-engine/issues/2)
- [#3 Persist ledger close timestamps](https://github.com/QuorumScope/quorumscope-engine/issues/3)
- [#4 Provide an always-on indexer deployment](https://github.com/QuorumScope/quorumscope-engine/issues/4)

## Reviewer links

- [Engine repository](https://github.com/QuorumScope/quorumscope-engine)
- [Console repository](https://github.com/QuorumScope/quorumscope-console)
- [Engine documentation](https://quorumscope.github.io/quorumscope-engine/)
- [Console documentation](https://quorumscope.github.io/quorumscope-console/)
- [Staging Console](https://quorumscope-console.vercel.app)
- [Staging Engine API](https://quorumscope-engine-api.onrender.com) (currently suspended; see Live
  Testnet verification above)
- [OpenAPI JSON](https://quorumscope-engine-api.onrender.com/openapi.json)
- [Latest release](https://github.com/QuorumScope/quorumscope-engine/releases/latest)
- [Deterministic demo](demo.md)
- [Security policy](../SECURITY.md)
- [Roadmap / open issues](https://github.com/QuorumScope/quorumscope-engine/issues)
