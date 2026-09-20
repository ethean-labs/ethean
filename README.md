# Ethean Lean Consensus Client

<div align="center">

![Ethean Logo](https://img.shields.io/badge/Ethean-Lean%20Consensus%20Client-blue?style=for-the-badge)

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg?style=flat-square)]()

**Rust consensus client for Ethereum Lean Consensus (Beam / leanEthereum)**

[Installation](#installation) · [Usage](#usage) · [Contributing](./CONTRIBUTING.md) · [Docs](./docs/readme/README.md)

</div>

## Overview

**Ethean** is a Rust **Lean Consensus** client for Ethereum’s post-quantum consensus
redesign (historically “Beam Chain” / leanEthereum). It is **consensus-only**: no
execution engine, no Beacon-chain compatibility layer, and no long-term BLS path.

### What we are building toward

Lean Consensus aims to replace today’s Beacon stack with:

- **Post-quantum signatures** — leanSig / XMSS-style hash-based signing instead of BLS
- **Aggregation** — leanMultisig today, later recursive aggregates and zkVM proofs (leanVM)
- **Faster finality** — seconds-scale 3SF (3SF-mini now; PQ heartbeat / Goldfish direction later)
- **Tighter slots and P2P** — ~4s slots, QUIC transport, Gossipsub evolution under huge validator counts
- **Lower stake floor** — roadmap target toward 1 ETH per validator, which stresses networking and aggregation

Ethean tracks [leanroadmap.org](https://leanroadmap.org/) research tracks and the
**pq-devnet** sequence (leanSpec / leanSig / leanVM pins), and treats peer clients
(Ream, Zeam, ethlambda, …) as **interop references**, not templates.

### What this repo does today

The `ethean` binary can already:

- Run under the **pq-devnet-4** label by default (offline smoke without bootnodes; dial when multiaddrs are set). **pq-devnet-5** stays a ready path.
- Advance **head / justified / finalized** in solo mode (local finality + aggregator), or resume a durable chain under `--data-dir`.
- Speak Lean **QUIC** req/resp (Status, blocks-by-root, blocks-by-range) and gossip admission paths as they harden.
- Export **`ethean_` Prometheus metrics**, `/healthz` `/readyz`, and Lean REST under `/lean/v1` (not Beacon `/eth/v1`).
- Lock behavior against **leanSpec fork-choice / STF fixtures** in `crates/spec-fixtures`.

Crypto and prove backends stay **fail-closed** until leanSig / leanVM production links are real.

### Key features

- **Lean-first workspace** — `ethean-types`, SSZ, genesis, state transition, 3SF-mini fork choice, validator duties, network, storage, RPC, metrics; wired by `ethean-node` + `bin/ethean`.
- **ChainOwner ownership** — one writer for head/sync; workers use snapshots and `ChainCommand`s.
- **Dual local modes** — durable fixed genesis (`--data-dir`) like peer pq-devnet packages, or ephemeral smoke (`--ephemeral`).
- **Operator-shaped networking** — network labels, bootnodes / fork-digest files, private mesh helpers; empty bootnodes mean offline, not a public join.
- **Observability** — scrape `:9100` by default; `--metrics` brings up Docker Grafana + Prometheus when Docker is available.
- **Spec alignment** — leanSpec FC/STF runners and fail-closed leanSig/leanVM gates instead of Beacon shortcuts.
- **House rules** — English-only tree, ≤300-line source files, no AI git attribution ([CONTRIBUTING.md](./CONTRIBUTING.md)).

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Installation](#installation)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Testing](#testing)
- [Contributing](./CONTRIBUTING.md)
- [License](#license)

## Architecture

Ethean is a **crate-per-concern** Rust workspace behind a single CLI.

| Layer | What lives here |
| --- | --- |
| Primitives / profile | Slots, forks, pinned `lstar`-style chain profile |
| Types + SSZ | Canonical Lean containers and encode/decode |
| Crypto | leanSig / XMSS surface (fail-closed without FFI) |
| Consensus | Genesis, state transition, **3SF-mini** fork choice |
| Validator | ~4s / interval duties, local aggregator role |
| Network | QUIC swarm, gossip admission, Status / blocks-by-root / range |
| Storage + sync | Durable SSZ / redb under `--data-dir`, sync gates |
| API + metrics | `/lean/v1`, `ethean_` gauges, health endpoints |
| Node shell | `ChainOwner` + start gates + `ethean` binary |

**Design intent:** stay aligned with leanSpec and pq-devnet operator pins; prefer Lean
behavior over Beacon convenience; keep gates honest when backends are missing.

Full crate map and open gates: [docs/readme/architecture.md](./docs/readme/architecture.md).
Migration plan library: [road-to/lean-consensus-migration/README.md](./road-to/lean-consensus-migration/README.md).

## Installation

1. Install [Rust](https://rustup.rs/) (pin: [`rust-toolchain.toml`](./rust-toolchain.toml)).
2. Clone this repo and build:

```bash
cargo build -p ethean --release
ethean version
```

The build publishes an `ethean` shim under `~/.cargo/bin`. Optional: Docker Desktop
for Grafana/Prometheus.

Details: [docs/readme/installation.md](./docs/readme/installation.md).

## Usage

Binary: `ethean`. Operational default network label: **pq-devnet-4** (D5 stays a
ready path). Without bootnodes the node runs offline under that label.

```bash
# Durable solo long-run (resume head under ./ethean-data)
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data

# Ephemeral smoke (new genesis each start)
ethean start --until-signal --network pq-devnet-4 --ephemeral

# Scrape :9100 by default; Grafana :3000 + Prometheus :9090 need Docker:
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data --metrics
```

Helpers: `./scripts/run-pq-devnet-4.sh` / `.\scripts\run-pq-devnet-4.ps1`.

Dual mode, private mesh, D5 ready path, flags, and troubleshooting:
[docs/readme/usage.md](./docs/readme/usage.md).

## API Documentation

Process scrape HTTP (default `127.0.0.1:9100`):

```bash
curl -s http://127.0.0.1:9100/healthz
curl -s http://127.0.0.1:9100/readyz
curl -s http://127.0.0.1:9100/metrics
```

Lean REST (default `:5052`, `/lean/v1/…` only — not Beacon `/eth/v1`):

```bash
curl -s http://127.0.0.1:5052/lean/v1/health
curl -s http://127.0.0.1:5052/lean/v1/chain/head
```

Full notes: [docs/readme/api.md](./docs/readme/api.md).

## Development

Lean workspace under `crates/`, binary in `bin/ethean`. Day-to-day:

```bash
cargo test
cargo clippy --workspace --all-targets
cargo fmt --check
```

Layout and conventions: [docs/readme/development.md](./docs/readme/development.md).
Contributor rules: [CONTRIBUTING.md](./CONTRIBUTING.md).

## Testing

```bash
cargo test
cargo test -p ethean-spec-fixtures
```

Coverage of modules and LeanSpec fixture runners:
[docs/readme/testing.md](./docs/readme/testing.md).

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for setup, code guidelines (300-line
files, English-only), git/PR rules, and no AI git attribution.

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

##  Acknowledgments

- **Ethereum Foundation**: For the Beam/Lean Chain specification
- **Rust Community**: For the excellent ecosystem
- **Contributors**: All developers who have contributed to this project
- **Lighthouse Team**: For inspiration and reference implementations

##  Support
- **Documentation**: [docs](./docs/)
- **Docs index**: [docs/README.md](./docs/README.md)
- **Folder READMEs and local conventions**: [docs/folder-readmes-and-local-conventions.md](./docs/folder-readmes-and-local-conventions.md)
- **Lean Consensus migration plans**: [road-to/lean-consensus-migration/README.md](./road-to/lean-consensus-migration/README.md) (active planning library; see also [road-to/README.md](./road-to/README.md))
- **Source tree**: [src/README.md](./src/README.md)
- **Lean Consensus tracks**: [leanroadmap.org research tracks](https://leanroadmap.org/#research-tracks)
- **Lean Consensus R&D (full site)**: [leanroadmap.org](https://leanroadmap.org/)
- **How we capture that locally**: [docs/leanroadmap-local-notes.md](./docs/leanroadmap-local-notes.md)
- **Source file size (300 lines)**: [docs/source-file-size-limit.md](./docs/source-file-size-limit.md)
- **Peer Lean clients (reference)**: [docs/peer-reference-clients.md](./docs/peer-reference-clients.md)
- **How Ream / ethlambda / Zeam run pq-devnets**: [docs/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md](./docs/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md)
- **Peer fixed genesis vs Ethean solo restart**: [docs/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md](./docs/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md)
- **Dual mode (persist + ephemeral)**: [docs/dual-mode-persist-and-ephemeral-2026-09-20.md](./docs/dual-mode-persist-and-ephemeral-2026-09-20.md)
- **Durable block prune (finalized − 256)**: [docs/durable-block-prune-finalized-keep-2026-09-20.md](./docs/durable-block-prune-finalized-keep-2026-09-20.md)
- **Durable persist / prune metrics**: [docs/durable-persist-prune-metrics-2026-09-20.md](./docs/durable-persist-prune-metrics-2026-09-20.md)
- **Grafana durable flush / prune panels**: [docs/grafana-durable-persist-prune-panels-2026-09-20.md](./docs/grafana-durable-persist-prune-panels-2026-09-20.md)
- **Configurable prune keep slots**: [docs/prune-keep-slots-config-2026-09-20.md](./docs/prune-keep-slots-config-2026-09-20.md)
- **Durable prune stall alert**: [docs/durable-prune-stall-alert-2026-09-20.md](./docs/durable-prune-stall-alert-2026-09-20.md)
- **Range-serve / serve-cache seed metrics**: [docs/range-serve-seed-metrics-2026-09-20.md](./docs/range-serve-seed-metrics-2026-09-20.md)
- **Grafana range-serve / seed panels**: [docs/grafana-range-serve-seed-panels-2026-09-20.md](./docs/grafana-range-serve-seed-panels-2026-09-20.md)
- **leanSpec FC finality / reorg / LMD**: [docs/leanspec-fc-finality-reorg-lmd-2026-09-20.md](./docs/leanspec-fc-finality-reorg-lmd-2026-09-20.md)
- **leanSpec FC safe-target + reorg_total**: [docs/leanspec-fc-safe-target-reorg-total-2026-09-20.md](./docs/leanspec-fc-safe-target-reorg-total-2026-09-20.md)
- **leanSpec FC prune votes not blocks**: [docs/leanspec-fc-prune-votes-not-blocks-2026-09-20.md](./docs/leanspec-fc-prune-votes-not-blocks-2026-09-20.md)
- **leanSpec FC extra suite**: [docs/leanspec-fc-extra-suite-2026-09-20.md](./docs/leanspec-fc-extra-suite-2026-09-20.md)
- **leanSpec FC tick safe snapshot gate**: [docs/leanspec-fc-tick-safe-snapshot-gate-2026-09-20.md](./docs/leanspec-fc-tick-safe-snapshot-gate-2026-09-20.md)
- **leanSpec FC finalized_safety empty-body gate**: [docs/leanspec-fc-finalized-safety-empty-body-gate-2026-09-20.md](./docs/leanspec-fc-finalized-safety-empty-body-gate-2026-09-20.md)
- **leanEthereum official repos plan**: [docs/lean-ethereum-official-repos-plan-2026-09-20.md](./docs/lean-ethereum-official-repos-plan-2026-09-20.md)
- **leanEthereum/pm indexed**: [docs/lean-ethereum-pm-indexed-2026-09-20.md](./docs/lean-ethereum-pm-indexed-2026-09-20.md)
- **FC MAX_ATTESTATIONS_DATA / D4 rate**: [docs/fc-max-attestations-d4-log-inv-rate-2026-09-20.md](./docs/fc-max-attestations-d4-log-inv-rate-2026-09-20.md)
- **leanMetrics safe_target / name map**: [docs/leanmetrics-safe-target-name-map-2026-09-20.md](./docs/leanmetrics-safe-target-name-map-2026-09-20.md)
- **leanSpec FC payload LMD weights**: [docs/leanspec-fc-payload-lmd-weights-2026-09-20.md](./docs/leanspec-fc-payload-lmd-weights-2026-09-20.md)
- **Full State SSZ encode/decode**: [docs/state-ssz-encode-decode-complete-2026-09-20.md](./docs/state-ssz-encode-decode-complete-2026-09-20.md)
- **Seven-client source research**: [docs/lean-peer-client-research-library-2026-09-19.md](./docs/lean-peer-client-research-library-2026-09-19.md)
- **Language (English only)**: [docs/english.md](./docs/english.md)
- **Commits (per file, English)**: [docs/commit-after-each-file.md](./docs/commit-after-each-file.md)
- **GitHub Issues**: [Report bugs](https://github.com/ethean-labs/ethean/issues)
- **Email**: support@Ethean.io

---

<div align="center">

**Built with ❤️ by the Ethean Team**

[Website](https://Ethean.io) • [GitHub](https://github.com/ethean-labs/ethean)

</div>

---

**Version**: tracked in root [`VERSION`](./VERSION) (currently synced to Cargo
workspace). After each development update run `.\scripts\bump-version.ps1`
(or `./scripts/bump-version.sh`) so the patch climbs `0.1.0` → `0.1.1` → …
→ `0.1.99` → `0.2.0`. See [docs/versioning.md](./docs/versioning.md)
([safe lock bump](./docs/bump-version-safe-cargo-lock-2026-09-20.md)).

**Note**: This is a development version. For production use, please wait for the stable release and conduct thorough testing in your environment.
