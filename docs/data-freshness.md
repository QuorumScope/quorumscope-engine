# Data Freshness & Compatibility

QuorumScope Engine tracks data freshness and protocol compatibility on every poll cycle. This state is exposed across `/api/v1/network`, `/api/v1/freeze-state`, `/api/v1/status`, and `/api/v1/preflight`.

## Freshness Model

The freshness model indicates whether the engine's view of freeze state can be relied upon for transaction safety checks.

| Field | Type | Description |
| --- | --- | --- |
| `status` | string | `current`, `indexing_behind`, `stale`, or `unknown` |
| `source_ledger` | integer | Ledger sequence number at which the current freeze state was observed |
| `latest_indexed_ledger` | integer | Highest ledger sequence processed by the indexer |
| `latest_network_ledger` | integer | Latest closed ledger sequence reported by the network RPC node |
| `ingestion_lag_ledgers` | integer | Difference between `latest_network_ledger` and `latest_indexed_ledger` |
| `observed_at` | string (ISO-8601) | Timestamp of the most recent network observation |
| `last_reconciled_ledger` | integer | Ledger sequence at which stored state was last reconciled against RPC |
| `last_reconciled_at` | string (ISO-8601) | Timestamp of last successful state reconciliation |

## Status Transitions

- **`current`**: The engine has observed the network within `STALE_AFTER_SEC` (default: 300s) and lag is at or below `MAX_LAG_LEDGERS` (default: 10 ledgers).
- **`indexing_behind`**: The indexer is actively polling, but its processed ledger trails the network ledger tip by more than `MAX_LAG_LEDGERS`.
- **`stale`**: No network observation has occurred within `STALE_AFTER_SEC`. This commonly happens when the indexer is stopped or the staging service is hibernating.
- **`unknown`**: No indexed state exists in the database.

## Compatibility Model

| Field | Type | Description |
| --- | --- | --- |
| `current_protocol_version` | integer | Protocol version reported by the RPC node (`getLatestLedger`) |
| `verified_protocol_max` | integer | Highest protocol version verified by this release (default: 28) |
| `compatibility` | string | `verified`, `unverified_protocol`, or `unknown` |

When `current_protocol_version > verified_protocol_max`, compatibility is reported as `unverified_protocol`. Downstream clients should present a compatibility warning indicating that newer protocol features have not been verified.
