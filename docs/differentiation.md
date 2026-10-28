# Differentiation

## The problem QuorumScope solves

CAP-77 introduces protocol-level Quorum Freeze state: validators can put specific ledger keys into
an on-chain freeze set, with an explicit transaction bypass list. Raw RPC/XDR state is authoritative,
but it is not an operator workflow. QuorumScope turns it into inspectable state, freshness and
compatibility status, deterministic preflight analysis, impact evidence, history, and operational
views.

This is Stellar-specific work because the Engine decodes Stellar CAP-77 config settings, Stellar
ledger keys, Stellar transaction envelopes and Soroban footprints, and applies the CAP-77 rules to
Stellar RPC responses. It is not a generic dashboard with a renamed chain selector.

## What it is not

QuorumScope is not:

- a general Stellar explorer;
- a wallet or transaction sender;
- a validator;
- a replacement for Stellar Core; or
- a generic monitoring dashboard.

It reads public RPC state and analyses candidate transaction XDR. It does not sign or submit
transactions and does not custody assets or private keys.

## Adjacent public project

The public search for CAP-77 and “quorum freeze” found one clearly adjacent implementation:

| Project | What it does | What QuorumScope does differently |
| --- | --- | --- |
| [Stellar-Guard](https://github.com/Virelis/Stellar-Guard) | Provides a UI-driven issuer/governance workflow that builds and submits unsigned/signed freeze XDR, tracks proposals and vote counts, and implements its own multisig governance contract. | QuorumScope is an inspection and analysis layer. It indexes protocol freeze state from Stellar RPC, preserves observed state transitions and evidence references, reports freshness and protocol compatibility, runs preflight against affected keys, and presents impact/history views. It does not create governance proposals or submit freezes. |

The distinction is about workflow boundary, not a claim that one project replaces the other. A
governance/execution tool and an independent protocol-state inspection tool can be used for different
parts of the same incident process.

The Stellar CAP-77 specification and the [SDF Quorum Freeze explanation](https://stellar.org/blog/foundation-news/quorum-freeze-cap-77-governed-onchain-incident-response)
are protocol references, not competing QuorumScope products.
