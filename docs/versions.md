# Version and compatibility status

Checked 2026-10-28 against the official Stellar software-version page and release notes.

## Network state

| Network | Current protocol | State | Source |
| --- | ---: | --- | --- |
| Stellar Mainnet | 29 | Active | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions) |
| Stellar Testnet | 29 | Active | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions) |

Protocol 29 is a security-focused release and introduces no new CAPs. The official compatibility
table lists XDR `v28.0`, Rust XDR `28.0.1`, Core `29.0.0`, and RPC `v29.0.0` for both networks.

## QuorumScope verified compatibility

`VERIFIED_PROTOCOL_MAX=29` is justified for the current CAP-77 workflow because:

- CAP-77 is a finalized Protocol 26 feature;
- Protocol 29 introduces no new CAPs;
- the pinned `stellar-xdr` 28.0.1 is the official Rust XDR line listed for Protocol 29;
- the Engine's frozen-key, frozen-key delta, bypass, and bypass-delta config-setting identifiers
  decode with that XDR; and
- the repository's unit, preflight, mocked-RPC/PostgreSQL, and live RPC checks cover the current
  decoding, hashing, trustline/AllowTrust, Soroban footprint, DEX, path-payment, claimable-balance,
  liquidity-pool, and API compatibility surfaces.

This verifies the implementation and current RPC response shape. It does not claim that a live
non-empty CAP-77 event has occurred on Testnet.

## Upcoming protocol

Bell / Protocol 30 releases are available, but Protocol 30 is not active on either network as of
this check. The official schedule lists the Testnet vote for 2026-11-03 and the Mainnet vote for
2026-11-23; either vote may fail or change. QuorumScope does not set its verified maximum to 30.

Protocol 30 readiness remains future work because CAP-84 adds a muxed-contract `SCAddress` variant
and CAP-88 adds millisecond-resolution ledger close-time XDR. Verification will be performed against
the activated network and toolchain, not inferred from release availability.

## Toolchain versions

These are the versions actually pinned or installed by this repository. “Pinned” is not the same as
“latest upstream,” and neither is a compatibility claim unless the verification statement above
explicitly covers it.

| Component | Installed or pinned value | What was verified |
| --- | --- | --- |
| Rust toolchain | 1.98.1 | Workspace build and tests use the repository toolchain |
| Rust edition | 2024 | Workspace compiles with the pinned toolchain |
| `stellar-xdr` | 28.0.1 | Protocol 29 XDR compatibility surface used by QuorumScope |
| PostgreSQL image | 16 | CI and local integration test configuration |
| API contract | OpenAPI 3.1.0 | Generated contract and Console snapshot match byte-for-byte |

Other dependency versions remain governed by `Cargo.toml` and `Cargo.lock`; this page does not label
them as newest upstream releases without a separate verification.
