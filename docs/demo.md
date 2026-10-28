# Deterministic non-empty demo

This is a **deterministic fixture demo**. It is not a live Testnet freeze, a real freeze incident,
or production evidence.

From the Engine repository, with Docker, Python 3, Rust, and `curl` installed:

```sh
./scripts/demo-nonempty.sh
```

The script starts a temporary PostgreSQL 16 container, generates realistic CAP-77 XDR with the
repository's Stellar XDR types, serves it from a local JSON-RPC fixture, runs the real indexer, and
starts the real API. It then queries the API for:

- one frozen account key with decoded key information and freeze freshness;
- one active bypass transaction hash;
- protocol 29 compatibility;
- impact data and evidence references;
- one blocked preflight transaction; and
- one transaction allowed by the configured bypass while retaining its finding.

The fixture follows the production chain through RPC decoding, PostgreSQL persistence, state
reconciliation, API handlers, and preflight analysis. It does not seed the database directly and it
does not add a production-only endpoint.

## Live Testnet verification

The live Testnet check verifies that QuorumScope can reach the current Soroban RPC, read the current
protocol observation, and reconcile the current CAP-77 configuration response. The observed live
freeze set is empty, so this does not prove a live non-empty incident.

## Deterministic non-empty demo

The local fixture proves the non-empty path, including frozen keys, bypass state, history-producing
indexer transitions, impact presentation, and preflight responses. The Console repository's
cross-repository check can run the same API through the TypeScript SDK and browser UI:

```sh
cd ../quorumscope-console
QS_DEMO_ENGINE_DIR=../quorumscope-engine ./scripts/demo-nonempty.sh
```

That command builds the SDK and Console against the local demo API, calls the API through the real
SDK, and checks the deployed-shaped Console routes and direct browser preflight path.

## Not yet live-proven

- a non-empty CAP-77 freeze set on public Testnet;
- live bypass, unfreeze, DEX, path-payment, trustline, claimable-balance, and Soroban-footprint
  behavior in one incident; and
- a production-grade always-on indexer deployment.
