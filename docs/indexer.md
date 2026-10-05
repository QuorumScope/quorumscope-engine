# QuorumScope Indexer

The QuorumScope Indexer is responsible for synchronizing CAP-77 freeze state from the Stellar Soroban RPC into the local PostgreSQL database. It tracks both `FrozenLedgerKeys` and `FreezeBypassTxs`.

## Modes

### One-Shot Mode
```bash
quorumscope index once
```
The one-shot mode retrieves the current configuration from the RPC exactly once, compares it against the local state, writes any deltas (freeze, unfreeze, bypass add, bypass remove) to the database, updates the ingestion checkpoint, and exits. This is useful for cron jobs or manual synchronization.

### Watch Mode
```bash
quorumscope index watch
```
The watch mode runs continuously, polling the RPC at a configured interval. It uses the same idempotent synchronization pipeline as one-shot mode but stays alive to keep the state continuously up-to-date.

## Checkpoint Behavior & Idempotency
The indexer keeps one checkpoint per network in `ingestion_checkpoints` for the `cap77_config` stream. Each poll reads `latestLedger` from `getLedgerEntries` and the network ledger and protocol version from `getLatestLedger`.

If the RPC ledger is older than the checkpoint, the poll only refreshes the network observation. Otherwise it diffs the configuration read from the network against `current_frozen_keys` and `current_bypasses`, and appends only real transitions to `freeze_changes` and `bypass_changes`. Reading the same state again changes nothing. A configuration snapshot is stored again only when its content changes or when a change needs it as evidence.

Each poll is one database transaction. It also opens a freeze episode when the first key is frozen, adds timeline events and an impact snapshot when the set changes, closes the episode when the set empties, and records a reconciliation when stored keys and bypasses equal what the network returned. If they do not match, the poll returns an error.

## Testing & Verification
- **Mocked RPC Tests**: We use `wiremock` to simulate the Soroban RPC in integration tests. These tests are clearly labeled as mocked tests and verify the deterministic behavior of the indexer (empty state, additions, removals) against a real PostgreSQL database.
- **Fixture Verified**: XDR parsing and domain models are validated against static, real-world fixtures to ensure correctness.
- **Live verified**: one-shot and watch runs against Soroban Testnet and a real PostgreSQL database are recorded in [verification.md](verification.md). `verify-live-rpc` checks the RPC connection alone.
