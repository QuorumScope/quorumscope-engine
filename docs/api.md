# HTTP API

Run `quorumscope serve --bind 127.0.0.1:8080` with `DATABASE_URL` set. The server applies migrations at startup. The contract is [openapi.json](../openapi/openapi.json), also served at `GET /openapi.json`. A test fails if the generated schema differs from the committed file.

Regenerate it with `cargo run -p quorumscope-api --example openapi`.

## Selecting a network

Every state endpoint takes an optional `network_id` UUID. Without it, the first network by name is used. An unknown network returns `404`.

## Endpoints

| Endpoint | Response |
| --- | --- |
| `GET /api/v1/network` | Network ID, name, passphrase, and freshness |
| `GET /api/v1/freeze-state` | Counts of active keys, bypasses, and episodes, plus freshness |
| `GET /api/v1/frozen-keys` | Frozen keys. Filters: `kind`, `active` (default `true`; `false` also lists keys no longer frozen) |
| `GET /api/v1/frozen-keys/{id}` | One key by 64-character hex hash, with decoded content, canonical XDR, and full change history |
| `GET /api/v1/bypasses` | Active bypasses |
| `GET /api/v1/incidents` | Freeze episodes |
| `GET /api/v1/incidents/{id}` | One episode |
| `GET /api/v1/incidents/{id}/timeline` | Events of one episode |
| `GET /api/v1/impact` | Stored impact evidence. See below |
| `GET /api/v1/status` | Indexer checkpoint and freshness. `404` until the indexer has run |
| `POST /api/v1/preflight` | Transaction preflight. See [preflight.md](preflight.md) |
| `GET /health/live`, `GET /health/ready` | Liveness, and database readiness (`503` when unavailable) |

## Freshness and compatibility

`network`, `freeze-state`, `status`, and `preflight` responses carry a `freshness` object.

| Field | Meaning |
| --- | --- |
| `status` | `current`, `indexing_behind`, `stale`, or `unknown` |
| `source_ledger` / `latest_indexed_ledger` | Ledger the stored freeze state was read at |
| `latest_network_ledger` | Latest ledger the indexer saw on the network |
| `ingestion_lag_ledgers` | `latest_network_ledger` minus `latest_indexed_ledger`, never negative |
| `observed_at` | When the indexer last read the network |
| `last_reconciled_ledger`, `last_reconciled_at` | Last poll where stored keys and bypasses matched what the network returned |
| `current_protocol_version` | Protocol version reported by `getLatestLedger` |
| `verified_protocol_max` | From `VERIFIED_PROTOCOL_MAX`, which the `quorumscope serve` command defaults to 28 |
| `compatibility` | `verified`, `unverified_protocol` (network protocol is newer than the maximum), or `unknown` (version or maximum missing; the maximum is missing only when the API is embedded without setting it) |

`stale` means no network observation within `STALE_AFTER_SEC`. `indexing_behind` means the indexer is observing but trails the latest ledger by more than `MAX_LAG_LEDGERS`. `unknown` means no indexed state exists. No close time is stored, so none is reported.

## Impact

`GET /api/v1/impact` returns two evidence classes that the indexer stores:

- `direct`: one record per active frozen key, with the ledger it has been frozen since.
- `protocol_derived`: frozen key counts recorded at ledgers where the freeze set changed.

`recently_observed`, `dependency_observed`, and `inferred` are part of the contract but need transaction history, which is not indexed. They never appear in `items`. The response lists them under `uncollected_evidence_classes` so clients can say so. Filters: `evidence_class`, `key_kind`, `key_id` (these two apply to direct records only). Every record carries an observation window with `is_complete_for_range: false`, because the indexer samples state by polling and does not read every ledger.

## Errors

```json
{
  "error": {
    "code": "invalid_input",
    "message": "id must be a UUID",
    "details": {},
    "request_id": "uuid"
  },
  "request_id": "uuid"
}
```

`request_id` appears inside `error` and at the top level. It is also the `x-request-id` header. The top-level copy predates the nested one and is kept for existing clients. `details` is always an object and is empty today. Codes: `invalid_input` (400), `not_found` (404), `method_not_allowed` (405), `payload_too_large` (413), `storage_error` (500), `storage_unavailable` (503), `internal_error`. Database errors are logged on the server and never returned.

Malformed transaction XDR is not an HTTP error. Preflight returns `200` with status `invalid_input`.

## Pagination

List endpoints use one-based `page` (default 1) and `page_size` (default 50, maximum 100), with a stable sort order, and return `{ "items": [...], "page": 1, "page_size": 50 }`. This is offset pagination, not keyset pagination. No total count is returned. If rows are added or removed while a client pages, items can be skipped or repeated. Keyset cursors were not added because the four list orderings differ and existing clients use `page`.

## CORS

No CORS headers are sent unless `ALLOWED_ORIGINS` lists origins. Listed origins may send `GET` and `POST` with a `Content-Type` header and can read `x-request-id`.

## Limits

The preflight body is limited to 256 KiB. Larger bodies return `413`.
