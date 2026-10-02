# Live Verification Evidence

This document records the exact output of a live verification run against the Soroban Testnet.

Note that this is a live network check, not a deterministic unit test.

- **Network:** Soroban Testnet
- **Date:** 2026-10-02
- **Command used:** `cargo run --bin verify-live-rpc`
- **Config setting queried:** FrozenLedgerKeys
- **Key:** AAAACAAAABE=
- **Latest ledger observed:** 4990637
- **Returned latest_ledger:** 4990638
- **Returned XDR:** AAAACAAAABEAAAAA

## Raw Output

```text
Starting LIVE RPC VERIFICATION against Soroban Testnet...
VERIFIED LIVE: Connected to Soroban Testnet
VERIFIED LIVE: Latest Ledger: 4990637
VERIFIED LIVE: Querying FrozenLedgerKeys config setting (Key: AAAACAAAABE=)
VERIFIED LIVE: Result returned latest_ledger=4990638
VERIFIED LIVE: Frozen set exists. Base64 XDR: AAAACAAAABEAAAAA
```
