# Ethean Lean Consensus Client

<div align="center">

![Ethean Logo](https://img.shields.io/badge/Ethean-Lean%20Consensus%20Client-blue?style=for-the-badge)

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg?style=flat-square)]()

**Rust consensus client for Ethereum Lean Consensus (Beam / leanEthereum)**

[Overview](#overview) · [Architecture](#architecture) · [Installation](#installation) ·
[Usage](#usage) · [API](#api-documentation) · [Development](#development) ·
[Testing](#testing) · [Contributing](./CONTRIBUTING.md) · [Docs](./docs/readme/README.md) ·
[License](#license)

</div>

## Overview

**Ethean** is a Rust **Lean Consensus** client for Ethereum’s post-quantum consensus
redesign (historically discussed as Beam Chain / leanEthereum). The product name
you see here is **Ethean Lean Consensus Client**; some crate paths may still carry
older packaging names treat those as history, not a second product.

Ethean is **consensus-only**. It does not run an execution engine, does not aim to
be a full L1 “everything node,” and does **not** treat today’s Beacon Chain (BLS
attestations, `/eth/v1` Beacon APIs, 12s mainnet habits) as the long-term target.
When Lean specs and Beacon convenience disagree, this repo prefers **Lean**.

### Why Lean Consensus exists

Ethereum’s current consensus layer works at mainnet scale, but the research
direction captured on [leanroadmap.org](https://leanroadmap.org/) is a deliberate
rewrite for a world where:

- **Signatures must survive quantum computers** : hash-based leanSig / XMSS-style
  schemes instead of long-lived BLS assumptions.
- **Validator sets may explode** : if the stake floor moves toward ~1 ETH, peer
  count and attestation volume jump; aggregation and P2P have to change shape.
- **Finality should be seconds, not minutes** : 3SF (and later PQ heartbeat /
  Goldfish-style ideas) instead of today’s longer confirmation story.
- **Slots get tighter** : interop discussion around ~4s slots pushes networking
  toward QUIC and evolving Gossipsub / set-reconciliation designs.
- **Roles may split** : attester–proposer separation (APS) and “rainbow staking”
  show up in the research tracks; clients need room for those duties.

That work is not a single repo: leanSpec, leanSig, leanVM, leanMultisig, leanMetrics,
pq-devnets, and several independent clients evolve together. Ethean’s job is to be
**one honest Rust client** in that set  interoperable where it matters, distinct
in layout and style.

### What Ethean is trying to become

In practical terms, Ethean should eventually:

1. **Speak the Lean wire and types** : canonical SSZ containers, fork-choice /
   state-transition behavior pinned to leanSpec (and operator digests on live meshes).
2. **Sign and verify with native leanSpec XMSS** : KoalaBear / Poseidon1 /
   SHAKE128 implemented in-tree, checked against leanSpec vectors and published
   leanSig keys; no BLS fallback anywhere.
3. **Aggregate with leanMultisig** : Type-1 aggregation of gossiped votes and
   Type-2 block proofs (leanVM `e2592df4`, the pq-devnet-4 pin), proved in a
   supervised `ethean-prover` process and verified in-process on every import.
4. **Run on pq-devnets like peer operators** : network labels, bootnode multiaddrs,
   fork digests, durable genesis packages, and metrics that operators can scrape.
5. **Stay operable solo** : local finality / aggregator smoke so contributors can
   advance head → justified → finalized without waiting for a public mesh.

Other Lean client implementations are useful **interop references** when wiring
wire formats or pq-devnet operator pins  not templates to copy crate layout or
style from. When peers disagree, leanSpec and the current pq-devnet pin win.
The reference list lives in
[docs/peer-reference-clients.md](./docs/peer-reference-clients.md).

### What this repo does today

The `ethean` binary is already useful for day-to-day Lean client work:

- **Network labels** : operational default **pq-devnet-4** (offline under that label
  when bootnodes are empty; dial when you paste QUIC multiaddrs). **pq-devnet-5**
  remains a prepared path (`config/networks/pq-devnet-5.*`, run scripts) for when
  operators publish a live mesh.
- **Solo chain progress** : with local finality and aggregator roles enabled by
  default, a long-run process advances **head / justified / finalized** without
  public peers. That is for development and Grafana, not a claim of mainnet safety.
- **Dual persistence modes** : `--data-dir` keeps a peer-like fixed genesis and
  resumes head after restart; `--ephemeral` rebuilds a recent genesis each start for
  fast smoke. See [docs/readme/usage.md](./docs/readme/usage.md).
- **Lean P2P surface** : QUIC swarm paths for Status, blocks-by-root, and
  blocks-by-range, plus gossip admission as it hardens against mesh digests.
- **HTTP that matches Lean, not Beacon** : scrape `:9100` (`/metrics`, `/healthz`,
  `/readyz`) and Lean REST under `/lean/v1` on `:5052`. There is no supported
  Beacon `/eth/v1` compatibility goal.
- **Spec fixtures** : `crates/spec-fixtures` locks fork-choice and related leanSpec
  vectors so regressions show up in `cargo test`, not only on a live mesh.
- **Observability** : process metrics are on by default; `ethean start … --metrics`
  can start Docker Compose Grafana + Prometheus when Docker Engine is available.

Honest limits matter: empty bootnodes mean **private / offline**, not “joined public
D4.” Aggregators and proposers need the `ethean-prover` binary. RocksDB and some transport
edges are still hardening. Treat Ethean as an **active Lean client under construction**,
aligned with research tracks, not a drop-in Beacon replacement.

### How development is organized

Work follows the Lean research surface and an in-repo migration library under
[`road-to/lean-consensus-migration/`](./road-to/lean-consensus-migration/README.md)
(phases for SSZ, genesis/clock, transition, fork choice, XMSS, leanVM, duties,
QUIC, storage, API, release). Session write-ups land in [`docs/`](./docs/README.md);
deep research extracts stay local and out of git.

Contributors are expected to keep the tree **English-only**, split source files at
**300 lines**, and leave **no AI / Cursor attribution** in commits or PRs
([CONTRIBUTING.md](./CONTRIBUTING.md), [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)).

### Key features

- **Lean-first workspace** : types, SSZ, genesis, state transition, 3SF-mini fork
  choice, validator duties, network, storage, RPC, and metrics, wired by
  `ethean-node` and `bin/ethean`.
- **ChainOwner ownership** : a single writer for head/sync; workers read snapshots
  and send `ChainCommand`s.
- **Dual local modes** : durable fixed genesis (`--data-dir`) or ephemeral smoke
  (`--ephemeral`), matching how peer clients think about pq-devnet packages.
- **Operator-shaped networking** : labels, bootnodes / fork-digest files, private
  mesh helpers; empty bootnodes are offline by design.
- **Observability** : `ethean_` Prometheus gauges on `:9100`; optional Docker
  Grafana / Prometheus via `--metrics`.
- **Spec alignment** : leanSpec FC/STF runners, native XMSS and verified
  leanMultisig block proofs instead of Beacon shortcuts.
- **House rules** : English-only tree, ≤300-line sources, clean human git history.

### Where to go next

| If you want… | Start here |
| --- | --- |
| Build and run | [Installation](#installation), [Usage](#usage), [docs/readme/usage.md](./docs/readme/usage.md) |
| Crate map | [Architecture](#architecture), [docs/readme/architecture.md](./docs/readme/architecture.md) |
| HTTP surfaces | [API Documentation](#api-documentation), [docs/readme/api.md](./docs/readme/api.md) |
| Contribute | [CONTRIBUTING.md](./CONTRIBUTING.md), [docs/readme/development.md](./docs/readme/development.md) |
| Lean R&D context | [leanroadmap.org](https://leanroadmap.org/), [docs/leanroadmap-local-notes.md](./docs/leanroadmap-local-notes.md) |

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
| Primitives / profile | Slots, forks, pinned `lstar` style chain profile |
| Types + SSZ | Canonical Lean containers and encode/decode |
| Crypto | Native leanSpec XMSS (KoalaBear / Poseidon1 / SHAKE128), batch verify |
| Aggregation | `ethean-multisig` (leanMultisig verify in-process) + `ethean-prover` process |
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
cargo build --release -p ethean -p ethean-prover
ethean version
```

The node finds `ethean-prover` next to its own binary (or via
`ETHEAN_PROVER_BIN`); aggregators and proposers need it. The build publishes an
`ethean` shim under `~/.cargo/bin`. Optional: Docker Engine with the compose
plugin for Grafana/Prometheus. Ethean targets Linux.

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

Helper: `./scripts/run-pq-devnet-4.sh`.

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
- **How Ream / ethlambda / Zeam run pq-devnets**: [docs/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md](./docs/peer-clients/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md)
- **Peer fixed genesis vs Ethean solo restart**: [docs/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md](./docs/peer-clients/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md)
- **Dual mode (persist + ephemeral)**: [docs/dual-mode-persist-and-ephemeral-2026-09-20.md](./docs/storage/dual-mode-persist-and-ephemeral-2026-09-20.md)
- **Durable block prune (finalized − 256)**: [docs/durable-block-prune-finalized-keep-2026-09-20.md](./docs/storage/durable-block-prune-finalized-keep-2026-09-20.md)
- **Durable persist / prune metrics**: [docs/durable-persist-prune-metrics-2026-09-20.md](./docs/observability/durable-persist-prune-metrics-2026-09-20.md)
- **Grafana durable flush / prune panels**: [docs/grafana-durable-persist-prune-panels-2026-09-20.md](./docs/observability/grafana-durable-persist-prune-panels-2026-09-20.md)
- **Configurable prune keep slots**: [docs/prune-keep-slots-config-2026-09-20.md](./docs/storage/prune-keep-slots-config-2026-09-20.md)
- **Durable prune stall alert**: [docs/durable-prune-stall-alert-2026-09-20.md](./docs/observability/durable-prune-stall-alert-2026-09-20.md)
- **Range-serve / serve-cache seed metrics**: [docs/range-serve-seed-metrics-2026-09-20.md](./docs/observability/range-serve-seed-metrics-2026-09-20.md)
- **Admin event backlog gauge**: [docs/admin-event-backlog-gauge-2026-09-25.md](./docs/observability/admin-event-backlog-gauge-2026-09-25.md)
- **Grafana range-serve / seed panels**: [docs/grafana-range-serve-seed-panels-2026-09-20.md](./docs/observability/grafana-range-serve-seed-panels-2026-09-20.md)
- **leanSpec FC finality / reorg / LMD**: [docs/leanspec-fc-finality-reorg-lmd-2026-09-20.md](./docs/lean-spec/leanspec-fc-finality-reorg-lmd-2026-09-20.md)
- **leanSpec FC safe-target + reorg_total**: [docs/leanspec-fc-safe-target-reorg-total-2026-09-20.md](./docs/lean-spec/leanspec-fc-safe-target-reorg-total-2026-09-20.md)
- **leanSpec FC prune votes not blocks**: [docs/leanspec-fc-prune-votes-not-blocks-2026-09-20.md](./docs/lean-spec/leanspec-fc-prune-votes-not-blocks-2026-09-20.md)
- **leanSpec FC extra suite**: [docs/leanspec-fc-extra-suite-2026-09-20.md](./docs/lean-spec/leanspec-fc-extra-suite-2026-09-20.md)
- **leanSpec FC tick safe snapshot gate**: [docs/leanspec-fc-tick-safe-snapshot-gate-2026-09-20.md](./docs/lean-spec/leanspec-fc-tick-safe-snapshot-gate-2026-09-20.md)
- **leanSpec FC finalized_safety empty-body gate**: [docs/leanspec-fc-finalized-safety-empty-body-gate-2026-09-20.md](./docs/lean-spec/leanspec-fc-finalized-safety-empty-body-gate-2026-09-20.md)
- **leanEthereum official repos plan**: [docs/lean-ethereum-official-repos-plan-2026-09-20.md](./docs/misc/lean-ethereum-official-repos-plan-2026-09-20.md)
- **leanEthereum/pm indexed**: [docs/lean-ethereum-pm-indexed-2026-09-20.md](./docs/misc/lean-ethereum-pm-indexed-2026-09-20.md)
- **FC MAX_ATTESTATIONS_DATA / D4 rate**: [docs/fc-max-attestations-d4-log-inv-rate-2026-09-20.md](./docs/lean-spec/fc-max-attestations-d4-log-inv-rate-2026-09-20.md)
- **leanMetrics safe_target / name map**: [docs/leanmetrics-safe-target-name-map-2026-09-20.md](./docs/observability/leanmetrics-safe-target-name-map-2026-09-20.md)
- **Node safe_target / reorg metrics**: [docs/node-safe-target-reorg-metrics-2026-09-20.md](./docs/observability/node-safe-target-reorg-metrics-2026-09-20.md)
- **Live ForkChoiceStore + safe-target attest**: [docs/live-fc-store-safe-target-attest-2026-09-24.md](./docs/lean-spec/live-fc-store-safe-target-attest-2026-09-24.md)
- **FC-driven head + durable rebuild**: [docs/fc-driven-head-durable-rebuild-2026-09-24.md](./docs/lean-spec/fc-driven-head-durable-rebuild-2026-09-24.md)
- **FC vote ingest (gossip/local)**: [docs/fc-vote-ingest-gossip-local-2026-09-24.md](./docs/lean-spec/fc-vote-ingest-gossip-local-2026-09-24.md)
- **Attest head vs safe-target + fork_choice API**: [docs/attest-head-vs-safe-target-fork-choice-api-2026-09-24.md](./docs/lean-spec/attest-head-vs-safe-target-fork-choice-api-2026-09-24.md)
- **What “external” backlog means**: [docs/what-external-backlog-means-2026-09-24.md](./docs/process/what-external-backlog-means-2026-09-24.md)
- **Admin events poll**: [docs/external-backlog-and-admin-events-poll-2026-09-24.md](./docs/networking/external-backlog-and-admin-events-poll-2026-09-24.md)
- **Identity version/ready + v0 ready**: [docs/identity-version-ready-v0-alias-2026-09-25.md](./docs/networking/identity-version-ready-v0-alias-2026-09-25.md)
- **Duties committee count + events pending**: [docs/duties-committee-count-events-pending-2026-09-25.md](./docs/networking/duties-committee-count-events-pending-2026-09-25.md)
- **Duties interval gating + monitoring SSE**: [docs/duties-interval-gating-monitoring-sse-2026-09-25.md](./docs/networking/duties-interval-gating-monitoring-sse-2026-09-25.md)
- **Hive v0 events alias**: [docs/hive-v0-events-alias-2026-09-25.md](./docs/networking/hive-v0-events-alias-2026-09-25.md)
- **Duties JSON attestation subnet**: [docs/duties-json-attestation-subnet-2026-09-25.md](./docs/networking/duties-json-attestation-subnet-2026-09-25.md)
- **Proposal duty + SSE events**: [docs/proposal-duty-sse-events-2026-09-25.md](./docs/networking/proposal-duty-sse-events-2026-09-25.md)
- **Validator duties JSON visibility**: [docs/validator-duties-json-visibility-2026-09-25.md](./docs/networking/validator-duties-json-visibility-2026-09-25.md)
- **FC store backlog**: [docs/dev-backlog-fc-store-2026-09-24.md](./docs/observability/dev-backlog-fc-store-2026-09-24.md)
- **leanSpec FC payload LMD weights**: [docs/leanspec-fc-payload-lmd-weights-2026-09-20.md](./docs/lean-spec/leanspec-fc-payload-lmd-weights-2026-09-20.md)
- **Full State SSZ encode/decode**: [docs/state-ssz-encode-decode-complete-2026-09-20.md](./docs/lean-spec/state-ssz-encode-decode-complete-2026-09-20.md)
- **Seven-client source research**: [docs/lean-peer-client-research-library-2026-09-19.md](./docs/peer-clients/lean-peer-client-research-library-2026-09-19.md)
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

**Version**: see [`VERSION`](./VERSION) (kept in sync with the Cargo workspace).
How to bump: [docs/versioning.md](./docs/versioning.md).
Release notes: [`CHANGELOG.md`](./CHANGELOG.md) · [GitHub Releases](https://github.com/ethean-labs/ethean/releases).

**Note**: This is a development version. For production use, please wait for the stable release and conduct thorough testing in your environment.
