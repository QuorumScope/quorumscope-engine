# Transaction preflight

`POST /api/v1/preflight` with `{ "transaction_xdr": "<base64 envelope>", "network_id": "<optional uuid>" }`.

Whitespace and line breaks in `transaction_xdr` are ignored. Transaction XDR is not logged or stored.

## What it does

The analysis lists the ledger keys the envelope names, hashes each canonical key, and compares the hashes with the stored freeze set. It does not simulate transaction application.

Keys examined:

- Transaction source account, and fee source account for fee bump envelopes.
- Operation source accounts, destinations, and the trustlines that Payment, PathPayment, offer, ChangeTrust, SetTrustLineFlags, Clawback, and CreateClaimableBalance operations name. Issuers have no trustline for their own asset.
- Soroban footprint keys, reported separately as read-only and read-write.

Operations that only touch their source account (SetOptions, ManageData, BumpSequence, Inflation) are checked through that account.

## Result

| `status` | Meaning |
| --- | --- |
| `blocked_validation` | A named key is in the freeze set. Deterministic. Each finding names the key and where it appears |
| `allowed_by_bypass` | The transaction content hash is in the active bypass set. Findings are still listed |
| `apply_time_risk` | A path payment chooses ledger entries during apply, and a freeze set exists. Conditional |
| `dex_conditional` | An offer operation depends on matched offers and trustlines, and a freeze set exists. Conditional |
| `unsupported_analysis` | An operation type is not analyzed beyond its source account. Insufficient information |
| `invalid_input` | The XDR could not be decoded |
| `state_unavailable` | Freshness is `stale` or `unknown`. No clear result is produced |
| `clear` | No finding. Only returned when freshness is `current` or `indexing_behind` |

The top-level status is the highest-priority finding, except that an active bypass makes it `allowed_by_bypass`. Priority from highest: blocked, apply-time risk, DEX conditional, unsupported, invalid, state unavailable. `confidence` is the weakest finding confidence: `deterministic`, `conditional`, or `insufficient_information`.

`source_ledger` and `freshness` say which stored state the result used. `freshness.compatibility` tells you when the network protocol is newer than the maximum this release was verified against.

## Assumptions to verify against the protocol

- The content hash is the standard Stellar transaction hash for the stored passphrase. For fee bump envelopes it is the hash of the fee bump transaction, and bypass matching uses that value. If CAP-77 matches the inner transaction instead, bypass results for fee bump envelopes would differ.
- Any footprint intersection is treated as blocked, whether read-only or read-write. The finding names which footprint it was in.
- Frozen keys of kinds other than account, trustline, contract data, and contract code are not stored.
