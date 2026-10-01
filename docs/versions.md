# Version verification

Checked 2026-10-01 21:00 UTC. These selections target Stellar Mainnet protocol 28 and Rust 1.98.1. A listed release is source verified; compatibility of the complete dependency graph still requires a successful Cargo build.

| Component | Selected version | Authoritative source | Selection |
| --- | --- | --- | --- |
| Stellar Mainnet protocol | 28 | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions) | Current Mainnet protocol |
| CAP-77 introduction | 26 | [Final CAP-77](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0077.md) | Protocol introduction |
| Stellar XDR | v28.0 | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions) | Current Mainnet XDR |
| `stellar-xdr` | 28.0.1 | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions), [28.0.1 crate release](https://github.com/stellar/rs-stellar-xdr/releases/tag/v28.0.1) | Newest stable patch on the protocol 28 line; the Stellar compatibility page still lists 28.0.0 |
| Stellar Core | 28.0.1 | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions), [release](https://github.com/stellar/stellar-core/releases/tag/v28.0.1) | Current stable protocol 28 release; the newer 29.0.0-internal tag is not a stable Mainnet release |
| Stellar RPC | 28.0.1 | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions), [releases](https://github.com/stellar/stellar-rpc/releases) | Current stable protocol 28 release |
| Stellar CLI | 28.1.0 | [Stellar software versions](https://developers.stellar.org/docs/networks/software-versions), [releases](https://github.com/stellar/stellar-cli/releases) | Current stable release |
| Rust stable | 1.98.1 | [Rust release announcements](https://blog.rust-lang.org/releases/), [1.98.1 announcement](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/) | Current stable patch release |
| Rust edition | 2024 | [Rust 2024 release announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/) | Supported by selected Rust |
| PostgreSQL | 18.6 | [PostgreSQL version policy](https://www.postgresql.org/support/versioning/), [18.6 notes](https://www.postgresql.org/docs/release/18.6/) | Current stable minor release of current stable major line |

Direct crate versions below were checked against each crate's `docs.rs/latest` page on the same date. These are the latest stable versions shown there, except `stellar-xdr`, whose `docs.rs/latest` page returned an older cached version. Cargo reported 28.0.1 as the newest release, and Stellar's crate release page confirms it was published on 2026-09-29 to enforce `VecM` length limits during Serde deserialization. This patch is on the same protocol 28 XDR line.

| Crate | Selected version | Authoritative source | Selection |
| --- | --- | --- | --- |
| `stellar-xdr` | 28.0.1 | [Stellar release](https://github.com/stellar/rs-stellar-xdr/releases/tag/v28.0.1) | Newest stable protocol 28 patch |
| `axum` | 0.8.9 | [docs.rs](https://docs.rs/axum/latest/axum/) | Latest stable |
| `tokio` | 1.53.1 | [docs.rs](https://docs.rs/tokio/latest/tokio/) | Latest stable |
| `reqwest` | 0.13.5 | [docs.rs](https://docs.rs/reqwest/latest/reqwest/) | Latest stable |
| `sqlx` | 0.9.0 | [docs.rs](https://docs.rs/sqlx/latest/sqlx/) | Latest stable |
| `tower-http` | 0.7.1 | [docs.rs](https://docs.rs/tower-http/latest/tower_http/) | Latest stable |
| `utoipa` | 6.0.0 | [docs.rs](https://docs.rs/utoipa/latest/utoipa/) | Latest stable |
| `serde` | 1.0.229 | [docs.rs](https://docs.rs/serde/latest/serde/) | Latest stable |
| `serde_json` | 1.0.151 | [docs.rs](https://docs.rs/serde_json/latest/serde_json/) | Latest stable |
| `tracing` | 0.1.44 | [docs.rs](https://docs.rs/tracing/latest/tracing/) | Latest stable |
| `tracing-subscriber` | 0.3.23 | [docs.rs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) | Latest stable |
| `clap` | 4.6.7 | [docs.rs](https://docs.rs/clap/latest/clap/) | Latest stable |
| `uuid` | 1.26.1 | [docs.rs](https://docs.rs/uuid/latest/uuid/) | Latest stable |
| `thiserror` | 2.0.21 | [docs.rs](https://docs.rs/thiserror/latest/thiserror/) | Latest stable |
| `anyhow` | 1.0.104 | [docs.rs](https://docs.rs/anyhow/latest/anyhow/) | Latest stable |
| `sha2` | 0.11.0 | [docs.rs](https://docs.rs/sha2/latest/sha2/) | Latest stable |
| `base64` | 0.23.1 | [docs.rs](https://docs.rs/base64/latest/base64/) | Latest stable |
| `hex` | 0.4.3 | [docs.rs](https://docs.rs/hex/latest/hex/) | Latest stable |
| `time` | 0.3.55 | [docs.rs](https://docs.rs/time/latest/time/) | Latest stable |

The installed `rustc` and Cargo defaults were 1.97.1 at inspection. Rust 1.98.1 is installed as a named toolchain and will be selected by `rust-toolchain.toml`.

Compatibility status: pending workspace dependency resolution and compile check. Do not treat this source verification alone as a passing build gate.
