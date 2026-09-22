# Changelog

All notable changes to **Ethean Lean Consensus Client** are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses the workspace version in [`VERSION`](./VERSION)
(`MAJOR.MINOR.PATCH`). GitHub Releases for milestone cuts live at
https://github.com/Pamenarti/Ethean/releases.

Session-level engineering notes remain under [`docs/`](./docs/). This file is
the curated operator-facing summary, not a dump of every working note.

## [Unreleased]

### Added

- Multi-platform `ethean` release archives (Windows x86_64, Linux x86_64/aarch64,
  macOS aarch64/x86_64) via `.github/workflows/release-binaries.yml`, with
  per-target `.sha256` files and a Binaries section on GitHub Release notes.

## [0.1.47] - 2026-09-20

Milestone covering patch work from `0.1.28` through `0.1.47` (current tip),
including durable chain data, operator metrics, leanSpec fork-choice coverage,
and a native leanSpec XMSS production backend.

### Added

- Durable applied-block persistence under the node data directory (`ethean-lc-d5-v1`
  schema), with dual-mode support for ephemeral and on-disk runs.
- Configurable block prune keep-slots, prune stall alerting, and Grafana panels
  for durable persist / prune / range-serve paths.
- Serve-cache seeding from the data directory so `blocks_by_range` / related
  handlers can answer from persisted history after restart.
- Gap warnings when range serve cannot fill a requested slot window.
- leanSpec fork-choice fixture locks for finality, tick-system, finalized
  safety, safe-target / `reorg_total`, prune-votes-not-blocks, and expanded
  extra suites.
- Node wiring for `safe_target` and reorg counters exposed on the metrics surface.
- Range-serve seed metrics and matching Grafana range-serve panels.
- Native leanSpec XMSS implementation in `ethean-crypto`: KoalaBear field,
  Poseidon1 (widths 16/24) with leanSpec round constants and circulant MDS,
  SHAKE128 PRF, aborting hypercube message hash with target-sum encoding,
  top/bottom Merkle trees with a sliding two-bottom-tree preparation window,
  and SSZ codecs for keys and signatures (no external crypto crates, no `unsafe`).
- Always-on `ProductionBackend`, `verify_batch` across cores, and
  `PublicKeyCache` for 52-byte validator public keys.
- Registry key path that decodes XMSS secret keys, derives the public key, and
  refuses a mismatched `pubkey_hex`.
- Indexed notes for leanEthereum official repos and leanEthereum/pm call material
  under `docs/`.

### Changed

- Operator plug-in and long-run observability paths assume durable data when a
  data directory is configured; prune behaviour is operator-tunable rather than
  hard-coded.
- `bump-version` scripts update only `ethean*` stanzas in `Cargo.lock` (avoids
  rewriting unrelated crates.io pins such as `tracing-attributes`).
- `test-aggregate` is no longer a default feature on shipped binaries; it remains
  available for `ethean-node` dev-dependencies, `ethean-leanvm-mock`, and an
  opt-in `ethean` feature for offline smoke runs.
- `leansig-backend` is retained as a no-op feature so existing run scripts keep
  building; the vendor leanSig patch and related PowerShell helpers were removed.
- Local proposer / attester install registry keys on the native backend;
  `ETHEAN_PRODUCTION_KEYGEN=1` is required for full PROD keygen at boot.

### Fixed

- Cargo.lock BOM / header-hash corruption when bumping versions on Windows.
- Grafana “no data” / epoch-1970 panel issues on long-run dashboards tied to
  range and durable metrics.

### Security

- Production attestation / block application no longer accepts keyless synthetic
  Type-2 proofs from a default `test-aggregate` feature in release builds.
- `LeanSigGate` reports ready against the native production scheme; registry
  keys that do not match the derived XMSS public key are rejected.

## [0.1.27] - 2026-09-20

Milestone covering patch work from `0.1.13` through `0.1.27`: Lean HTTP and
QUIC surfaces, Hive client packaging, validator duties, aggregation honesty,
leanVM IPC, and the first operator plug-in readiness surface.

### Added

- Lean HTTP API listener on port `5052` (`RpcListen` / node HTTP shell) for
  operator and interop tooling.
- Fixed QUIC listen port wiring for mesh dial and local pq-devnet style runs.
- Hive client Docker scaffold and upstream `clients/ethean` drop-in path, plus
  Hive lean config consume and `leansig` feature wiring for CI-style runs.
- ACC profile and attestation subnet wiring for gossip admission.
- Local attester duty path and owned-index proposer gate.
- Registry private-key load for local signing material.
- Type-1 attest-before-prove honesty checks (empty / incomplete Type-1 paths
  fail closed where required).
- leanVM IPC split request/response operations and versioned frame codec
  towards external prover sidecars.
- Operator plug-in readiness surface so a node can report whether storage,
  crypto, signer, network, and prover gates are ready before serving traffic.
- Fork-choice coverage for `blockWeights` snapshots and aggregated payload
  pool snapshots on top of the earlier STF/FC runners.

### Changed

- Default networking posture moves from compile-only scaffolding toward a
  runnable QUIC + HTTP operator shell aimed at pq-devnet plug-in checklists.
- Hive and leanSpec fixture consumers share more of the same Lean config and
  feature flags used by the binary.

### Fixed

- Hive / Docker packaging gaps that blocked an upstream-style `clients/ethean`
  drop-in (image layout and feature flags aligned with leanSig-capable builds).

## [0.1.12] - 2026-09-20

First public milestone: Lean workspace identity, typed Lean containers, and a
green leanSpec state-transition / fork-choice fixture lane. Includes foundation
work that landed before the root `VERSION` file (`0.1.1`) through `0.1.12`.

### Added

- Project identity as **Ethean Lean Consensus Client** (rename from the earlier
  Panro Beacon packaging) with a virtual Cargo workspace (`ethean-primitives`,
  `ethean-profile`, `ethean-ssz`, `ethean-types`, `ethean-crypto`,
  `ethean-genesis`, `ethean-transition`, `ethean-fork-choice`, `ethean-storage`,
  `ethean-network`, `ethean-sync`, `ethean-validator`, `ethean-node`,
  `ethean-rpc`, `ethean-metrics`, `bin/ethean`, and related crates).
- Lean SSZ / lstar-oriented container types; Beacon node types and BLS-centric
  long-term API assumptions retired from the production path.
- Genesis / clock scaffolding and fork-choice crate integration for Lean slot
  and justification semantics.
- leanSpec fork-choice rejection runner, tick/import fixture runner, attestation
  and block-body attestation decode runners, gossip aggregate FC runner,
  wall-clock tick and justification coverage, earliest-admissible tick checks.
- STF fixture runner scaffold and a full state-transition fixture suite green.
- Early Hive leanSpec fixture consumer scaffold and org-profile README draft.
- Root `VERSION` file synced to `[workspace.package] version`, with patch-bump
  scripts for PowerShell and shell.
- Migration library under `road-to/lean-consensus-migration/` (phases 00–13
  planning, release gates, upgrade/rollback runbooks) and `tools/release/`
  repro-build / legacy-scan scaffolding.
- Storage schema id `ethean-lc-d5-v1` with a forward-only version gate;
  metrics schema versioning (`ethean-metrics-v2`) with high-cardinality label
  name bans.

### Changed

- Consensus target is Lean / leanEthereum interop (pq-devnet direction), not
  Beacon Chain convenience APIs (`/eth/v1` is not the long-term surface).
- Workspace dependency set trimmed of unused BLS / Beacon HTTP pins on the
  production path as Lean crates landed.

### Security

- Crypto isolation and XMSS leaf-safety gates documented as non-waivable for
  promotion ([docs/release/gates.md](./docs/release/gates.md)).
- Legacy symbol scans under `tools/release/` treat remaining `blst` /
  `blstrs` / `bls12_381` under `crates/` or `bin/` as hard-fail for release
  promotion.

[Unreleased]: https://github.com/Pamenarti/Ethean/compare/v0.1.47...HEAD
[0.1.47]: https://github.com/Pamenarti/Ethean/compare/v0.1.27...v0.1.47
[0.1.27]: https://github.com/Pamenarti/Ethean/compare/v0.1.12...v0.1.27
[0.1.12]: https://github.com/Pamenarti/Ethean/releases/tag/v0.1.12
