# Known Limitations

This document provides a factual record of known limitations, protocol boundaries, and verification constraints in QuorumScope Engine.

## Protocol Boundaries

### 1. No Live Non-Empty Freeze Set Observed
Stellar Testnet currently operates with an empty freeze set (0 frozen keys and 0 bypass entries). As a result:
- Ingestion of non-empty freeze sets has only been verified against wiremock RPC simulations and constructed test fixtures.
- Live RPC interaction with real network freeze events will be verified when such events occur on public Stellar networks.

### 2. Live non-empty state
- The engine verifies the current Protocol 29 CAP-77 compatibility surface, but Stellar Testnet has
  not supplied a live non-empty freeze set in the observed checks.
- The deterministic fixture demo covers non-empty decoding, persistence, API presentation, and
  preflight without presenting synthetic data as live evidence.

### 3. Protocol 30 readiness
- Protocol 30 is upcoming, not active. CAP-84 and CAP-88 require a future XDR and activated-network
  verification pass.

## Ingestion & Storage Boundaries

### 3. Uncollected Impact Classes
The OpenAPI contract specifies five evidence classes for impact analysis:
- `direct`: Implemented and stored (active frozen keys).
- `protocol_derived`: Implemented and stored (freeze set changes at specific ledgers).
- `recently_observed`: Not collected (requires transaction history indexing).
- `dependency_observed`: Not collected (requires smart contract call graph indexing).
- `inferred`: Not collected (requires heuristic network analysis).

The API explicitly returns uncollected classes in `uncollected_evidence_classes` so clients can distinguish between absent records and uncollected categories.

### 4. Ledger Close Times
Ledger close timestamps are not stored or indexed in the database. Timestamps reported in API responses reflect the time of engine observation rather than on-chain consensus timestamps.

## Infrastructure Boundaries

### 5. Staging Environment Hibernation
The staging instance runs on Render free tier hosting:
- Inactivity triggers automatic service hibernation.
- Cold starts take approximately 30 to 60 seconds.
- During initial wake-up, data freshness may report as stale until the first polling cycle completes against the testnet RPC.

### 6. Security Audit Status
QuorumScope Engine has not undergone a formal third-party security audit.
