# Transaction preflight

`POST /api/v1/preflight` with `{ "transaction_xdr": "<base64 envelope>", "network_id": "<optional uuid>" }`.

Whitespace and line breaks in `transaction_xdr` are ignored. Transaction XDR is not logged or stored.

## What it does

The analysis lists the ledger keys the envelope names, hashes each canonical key, and compares the hashes with the stored freeze set. It does not simulate transaction application. The rules follow [CAP-77](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0077.md) (protocol 26). See the review section at the end.

Keys examined:

- Source account of the transaction, of each operation, and the fee source of a fee bump envelope (and the inner transaction's source).
- Source trustlines for Payment, PathPaymentStrictReceive and Send (`sendAsset`), ManageSellOffer, ManageBuyOffer, CreatePassiveSellOffer (selling and buying assets), ChangeTrust, and CreateClaimableBalance.
- Destinations: for Payment and PathPayment, the destination account when the asset is native and the destination trustline when it is a credit asset. CreateAccount and AccountMerge name an account. AllowTrust, SetTrustLineFlags and Clawback name a trustline. RevokeSponsorship names the entry or signer account.
- Soroban footprint keys, reported separately as read-only and read-write.

Issuers have no trustline for their own asset.

## Result

| `status` | Meaning |
| --- | --- |
| `blocked_validation` | A named key is in the freeze set. The protocol rejects the transaction at validation. Deterministic. Each finding names the key and where it appears |
| `allowed_by_bypass` | The transaction content hash is in the active bypass set and that is the only reason it is not blocked. Blocked findings are still listed |
| `apply_time_risk` | ClaimClaimableBalance, LiquidityPoolDeposit, or LiquidityPoolWithdraw is present and a freeze set exists. These fail at apply time if they touch a frozen trustline or account. A bypass does not cover them. Conditional |
| `dex_conditional` | An offer or path payment operation is present and a freeze set exists. While matching, the protocol removes an offer whose owner or trustline is frozen and keeps matching. The transaction does not fail for that reason. A bypass does not cover it. Conditional |
| `unsupported_analysis` | An operation type is not analyzed beyond its source account (BeginSponsoringFutureReserves, EndSponsoringFutureReserves, ClawbackClaimableBalance). Insufficient information |
| `invalid_input` | The XDR could not be decoded |
| `state_unavailable` | Freshness is `stale` or `unknown`. No clear result is produced |
| `clear` | No finding. Only returned when freshness is `current` or `indexing_behind` |

Precedence from highest: blocked, apply-time risk, DEX conditional, unsupported, invalid, state unavailable. When the hash is bypassed and blocked findings exist, the blocked findings are set aside and the rest decide the result, so apply-time, DEX, and unsupported findings are still reported. `allowed_by_bypass` is returned only when nothing else remains. A bypass hash for a transaction that touches no frozen key changes nothing: `is_bypassed` is true and the status is whatever the findings give.

`confidence` is the weakest finding confidence: `deterministic`, `conditional`, or `insufficient_information`.

`source_ledger` and `freshness` say which stored state the result used. `freshness.compatibility` tells you when the network protocol is newer than the maximum this release was checked against.

## Review against CAP-77

Reviewed on 2026-10-05 against `core/cap-0077.md` in `stellar/stellar-protocol` (protocol version 26), read directly from the repository. The review is by reading the specification. It was not checked on a live network, because testnet had no frozen keys or bypass entries.

Confirmed by the text:

- Footprint: a Soroban transaction is invalid if any key in the read-only or read-write footprint is frozen. Soroban transactions are those with a single `InvokeHostFunction`, `ExtendFootprintTtl`, or `RestoreFootprint` operation.
- Source accounts of the transaction, fee bump transaction, or any operation make the transaction invalid.
- The source-trustline and destination lists above match the CAP's lists.
- Bypass entries are transaction content hashes, which omit signatures. For a fee bump, the outer transaction's hash is used so the fee source is checked. The CAP gives this as the reason the inner hash is not used. This engine hashes the signature payload of the outer transaction for the stored network passphrase.
- The bypass applies at validation time only. It is ignored for the apply-time checks on claimable balance and liquidity pool operations and for DEX handling.
- DEX: an offer that would change a frozen entry is removed and matching continues.
- Frozen key types are account, trustline, contract data, and contract code. Pool share and issuer trustlines cannot be frozen.

Still open:

- The CAP does not state whether the content hash includes the network ID. The engine uses the standard Stellar transaction hash (signature payload with network ID). This has not been checked against stellar-core.
- The CAP names apply-time failures for claimable balance claims and pool deposit or withdraw only when they modify a frozen entry. QuorumScope cannot tell which entry, so it reports a conditional result whenever a freeze set exists. The same applies to DEX operations. Narrowing these would need ledger state the engine does not store.
- ManageData, SetOptions, BumpSequence, and Inflation are treated as touching only their source accounts. The CAP's lists do not name them, but the CAP does not say it lists every access.
- The CAP exceptions (offers may be removed, and entries sponsored by a frozen account may be removed) are not modeled.
