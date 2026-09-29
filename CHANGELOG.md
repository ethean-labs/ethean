# Changelog

All notable changes to **Ethean Lean Consensus Client** are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses the workspace version in [`VERSION`](./VERSION)
(`MAJOR.MINOR.PATCH`). GitHub Releases for milestone cuts live at
https://github.com/ethean-labs/ethean/releases.

Session-level engineering notes remain under [`docs/`](./docs/). This file is
the curated operator-facing summary, not a dump of every working note.

## [Unreleased]

### Changed

- Proposers build, sign and prove the next slot's block during the last
  interval of the current slot and hold it until the slot starts (same pattern
  as ethlambda and Grandine lean); a slot already being proved is never
  re-planned or re-signed.
- The mesh duty loop keeps the swarm pumped between intervals and collects
  finished proofs immediately instead of sleeping until the next interval.
- Majority Status tip follows leanSpec `get_network_finalized_slot` (ties go
  to the higher slot, then the most-reported root); peers stop voting once
  their last connection closes.
- `config.yaml` `LOG_INV_RATE` is parsed and a mismatch with the built-in rate
  is logged.
- Installed XMSS keys build their next bottom tree on a background thread
  (a quarter of the cores) once the slot reaches the right half of the window,
  so signing no longer stalls about 10 s every 65536 slots.
- Gossip vote signatures of each network pump window are verified in one
  parallel batch before in-order admission (about 10× faster than serial on
  20 cores).
- Block proofs are submitted before aggregation work in the same duty step.
- Aggregators prove pooled votes once per slot at interval 2, as leanSpec
  `tick_interval` does, instead of on every interval; on a shared host the
  extra Type-1 jobs slowed every node's block proof.
- Proposed blocks carry at most 1 attestation data by default
  (`--max-block-attestation-data`, 0–8): the Type-2 merge time steps with the
  next power of two of its component count, the spec maximum needs about
  12 s or more to prove, and with cap 3 proofs overran the 4 s slot often
  enough to fork the chain and stall finality. Under a cap the data with the
  widest proof coverage wins for each target slot.
- Fork-choice safe target also counts single gossip votes newer than the
  pending aggregates (deviation from leanSpec), so non-aggregators no longer
  lag the aggregator's safe target and split votes across targets.
- CI runs `cargo fmt --check`, `clippy -D warnings` and the `ethean-node` /
  `ethean-spec-fixtures` tests; the workspace is formatted and clippy-clean.
- Local votes use leanSpec `get_attestation_target` (lookback toward the safe
  target, then back to a slot justifiable after finalization) and the head
  state's justified source; votes for unjustifiable targets never counted.
- A finished block proof is published while its parent is still in the
  fork-choice tree and no block at or past its slot became head, instead of
  being dropped whenever the head moved.
- Below the spec maximum, block building skips votes that change no
  justification bookkeeping, so stale pool data cannot fill the data cap.

### Fixed

- The fork-choice store was never created on a `config.yaml` genesis (the
  sealed genesis header failed the anchor check), so mesh nodes ran without LMD
  head, safe target or justification.
- A peer that dialed us while we dialed it was forgotten when the duplicate
  connection closed; Status requests to it failed every second.

- XMSS key generation on Windows: `OsRandom` read `/dev/urandom` directly and
  now uses `getrandom`.
- A prover queue full of aggregation and split jobs no longer refuses the
  block proof.
- The leanMultisig verifier is compiled on a background thread at start; the
  first block or aggregate verification used to stall the duty loop for 5–10 s.

- `/lean/v0/health` returns exactly the leanSpec body (`status`, `service`);
  the extra `version` field failed the leanSpec API endpoint fixture.

### Added

- `ethean devnet-init` writes a lean-quickstart shaped bundle (config.yaml,
  validators.yaml, PROD XMSS keys, nodes.yaml, node keys) and
  `scripts/local-devnet.ps1` runs N proving nodes on one host without Docker;
  three nodes finalize.
- Grandine lean (`grandinetech/lean`) added to the peer reference list.

- Majority Status finalized tip for catch-up and duty horizon so a lone ahead
  adversarial peer cannot hijack sync (Hive bad-checkpoint rejection path).
- Head-behind-finalized catch-up pins the finalized root; local Hive smoke
  script `tools/hive/smoke-local.ps1`.
- Proof-service priority queue (Block > Attestation > Split), defer Split while
  a block proof is in flight, and mesh Type-2 request retries on intervals 0..=2.
- leanSpec prod-scheme genesis state / sealed header root pins for 1 and 4
  validators (`PROD4_GENESIS_*`, `PROD1_GENESIS_BLOCK_ROOT`).
- Late-joiner sync: monotonic peer horizon, duty ticks observe chain head,
  Status tip follow-up range/root after each block response, horizon uses
  `max(head, finalized)` (`docs/networking/late-joiner-sync-catchup-2026-09-25.md`).
- Dependabot weekly updates for Cargo and GitHub Actions; Hive GHCR probe
  (`tools/hive/check-ghcr.ps1`); Docker workflow best-effort public package
  visibility for anonymous Hive pulls; `hive_rpc_compat_v0_surface` route lock.
- `--checkpoint-sync-url` bootstraps from a peer's `/lean/v0` finalized state
  and block pair after verifying they belong together (leanSpec checkpoint
  sync); failed fetch aborts startup; the last leanMetrics series are recorded:
  connected and mesh peers by client family through libp2p identify, and
  fork-choice reorg depth.
- `crates/spec-fixtures` runs every case of the leanSpec ssz, networking codec,
  slot clock, justifiability, Poseidon, sync, single-message proof and API
  endpoint suites, in addition to the full fork-choice and state-transition
  drives; unsupported vectors are listed, not hidden. The leanSpec node
  registry's attestation coverage gauges are exported next to the leanMetrics
  table.
- Hive `test_driver` routes (`fork_choice/init`, `fork_choice/step`,
  `state_transition/run`, `verify_signatures/run`) behind
  `HIVE_LEAN_TEST_DRIVER=1`, driving the production fork choice, state
  transition and proof verification; every extracted leanSpec fixture passes.
- Hive `/lean/v0` HTTP surface (`health`, `checkpoints/justified`, `fork_choice`,
  `states/finalized`, `blocks/finalized`, `admin/aggregator`) with `/lean/v1`
  aliases, HTTP/1.1 request bodies, and a published fork-choice snapshot
  (node weights stay 0 until `ForkChoiceStore` is wired on the chain owner).
- Hive / lean-quickstart `ethean start` aliases: `--network` may be a `config.yaml`
  path, `--node-key` / `--private-key-path` (secp256k1 hex), `--socket-address` /
  `--socket-port`, `--is-aggregator`, `--aggregate-subnet-ids`,
  `--attestation-committee-count`, `--checkpoint-sync-url`, `--validator-registry-path`, and
  `--observability-stack` for Grafana compose. `--metrics` is a no-op keep-on for
  scrape HTTP. Env fallbacks `ETHEAN_HTTP_ADDRESS` and `ETHEAN_METRICS_ADDRESS`.
- secp256k1 libp2p swarm identity: persist `<data-dir>/node.key` across restarts,
  generate per run with `--ephemeral`, and log the PeerId plus dialable
  `/ip4/…/udp/…/quic-v1/p2p/<peerid>` multiaddr. `/lean/v1/node/identity` uses
  that PeerId.
- `--bootnodes` accepts `none`, CSV of QUIC multiaddrs and/or `enr:` records, or
  a YAML `nodes.yaml` list. secp256k1 ENRs decode to a QUIC multiaddr
  (`quic` port, fallback `udp`).
- Multi-arch Docker image (`linux/amd64` + `linux/arm64`) via
  `.github/workflows/docker-image.yml`, published to
  `ghcr.io/<github-owner>/ethean` (lowercased). Tags: `sha-<short>` every
  build, `unstable` on `master`, semver plus `latest` / `devnet5` /
  `latest-devnet5` on `v*` tags. Runtime image ships `ethean` and
  `ethean-prover` in `/usr/local/bin` with `config/networks/` under `/app`.
- Hive drop-in `docker/hive/upstream-clients-ethean/` matching
  `hive/clients/ream/` (Dockerfile wrapping `ghcr.io/ethean-labs/ethean:devnet5`,
  `Dockerfile.git`, `ethean.sh`, `hive.yaml`, `validators.yaml`).
- Repository URL in `Cargo.toml`, release-binaries `REPO_URL`, and changelog
  compare links set to `https://github.com/ethean-labs/ethean`.

### Changed

- Fork choice rejects votes from validators outside the target state's
  registry (`VALIDATOR_NOT_IN_STATE`) and the driver path verifies vote
  signatures and aggregate proofs before admission.
- Block production follows leanSpec vote selection: target-slot order, known
  head roots, justified-source and on-chain checks, re-anchoring rounds, and
  a body cap of `MAX_ATTESTATIONS_DATA` (8, was 16).
- Aggregators recover each block vote's Type-1 proof from the block proof
  (`split_type2`) and reuse it as a merge child; known on-chain payloads feed
  `lean_latest_known_aggregated_payloads`.
- Wire layer now follows leanSpec lstar: gossip fork segment `12345678`,
  anonymous gossipsub with the spec 20-byte message id and mesh parameters, SSZ
  `Status`, varint + snappy-framed req/resp chunks with spec response codes and
  request encodings, subnet count from `ATTESTATION_COMMITTEE_COUNT`.
- Genesis header `body_root` is `hash_tree_root(BlockBody([]))` as in leanSpec
  (was zero), and `is_justifiable_after` uses an exact integer square root.
- Console logs disable ANSI when stdout is not a terminal or `NO_COLOR` is set.

### Fixed

- A gossip publish with no mesh peer (`InsufficientPeers`) is dropped with a
  warning in every mode; it used to stop the node unless `--local-finality`
  was set (hive checkpoint-sync scenarios).
- Checkpoint sync re-anchors the live fork-choice store on the fetched pair, so
  `/lean/v0/fork_choice` shows the checkpoint as justified/finalized with no
  pre-anchor nodes (hive checkpoint-sync scenarios).
- Docker image build copies `vendor/` before `cargo chef cook` so the leanVM
  Windows `[patch]` overlays resolve inside the builder stage.
- Wall-clock duty loops wait for genesis (hive and lean-quickstart start the
  client before `GENESIS_TIME`) instead of shutting down on the first tick.
- SSZ wire format now matches leanSpec: `BlockBody` and `MultiMessageAggregate`
  carry their container offset, `State.validators` is a plain concatenation,
  `Validator.index` is unbounded at decode time, and signature roots use the
  `Signature` container layout. Blocks and served states produced before this
  change were undecodable by spec clients. Durable schema id is now
  `ethean-lc-d5-v2`; recreate `--data-dir` trees.

- Windows `x86_64-pc-windows-msvc` release archives compile via
  `vendor/leanvm-windows` overlays for leanVM `system-info` / `zk-alloc`
  (Unix `getrusage` / sparse `mmap` were blocking the Windows CI job).

## [0.1.53] - 2026-09-24

Milestone covering patch work from `0.1.48` through `0.1.53`: leanMultisig
proving, leanMetrics v3, a live fork-choice store that owns tip and safe-target,
vote ingest, and Lean HTTP surfaces for fork_choice and admin events.

### Added

- leanMetrics standard metrics (`lean_*`, metrics schema `ethean-metrics-v3`):
  all 63 metrics of leanEthereum/leanMetrics with matching types, labels and
  buckets, recorded across signing, aggregation, block production, state
  transition, gossip and peers; Grafana provisions the leanMetrics interop
  dashboard. Existing `ethean_*` metrics are unchanged.
- leanMultisig aggregation at leanVM `e2592df4` (the pq-devnet-4 pin shared with
  ream, ethlambda and zeam): in-process Type-1 / Type-2 verification
  (`ethean-multisig`) and proving in the new `ethean-prover` process.
- Aggregator duty: verified `SignedAttestation` votes are aggregated into
  Type-1 proofs and published on the aggregation topic.
- Proposals carry a merged Type-2 block proof (body attestation proofs, then the
  proposer's signature over the block root).
- Release archives for Linux x86_64/aarch64, macOS aarch64/x86_64 and Windows
  x86_64 via `.github/workflows/release-binaries.yml`, each containing `ethean`
  and `ethean-prover`, with per-target `.sha256` files and a Binaries section on
  GitHub Release notes. Windows uses `vendor/leanvm-windows` overlays for leanVM
  `system-info` / `zk-alloc` (proving arena disabled; System allocator).
- Live optional `ForkChoiceStore` on `ChainOwner`: init after genesis / durable
  restore, `fc_on_block` / `fc_on_tick`, and interval-3 safe-target for local
  attestations.
- FC-driven canonical tip: import any parent known to the store, orphan/drain
  sync against the FC parent set, rebuild the store from durable genesis and
  block blobs after `--data-dir` resume.
- Verified gossip and local attestation votes feed the live store
  (`on_attestation_data` / `on_aggregated_attestation`).
- `GET /lean/v1/chain/fork_choice` JSON snapshot (live flag, roots, reorg_total,
  block and vote-pool sizes).
- `GET /lean/v1/events` JSON poll drain for redacted admin events
  (`DutySuppressed`, `HeadUpdated`, `HeadSlot`, `Readiness`).
- Solo local-finality path persists applied blocks so `--data-dir` resume keeps
  them across restarts.

### Changed

- Gossip blocks are imported only after their block proof verifies against the
  parent state's registry (leanSpec `verify_signatures`); unsigned `Block`
  payloads are no longer accepted.
- Attestation subnets carry `SignedAttestation` and every vote's XMSS signature
  is verified; aggregates are verified against their participants' keys.
- Local attestations use the fork-choice tip as **head** and interval-3
  safe-target as **target** when the store is live (no longer voting
  safe-target for both fields).
- When the FC store is live, local-finality smoke checkpoint promotion is
  skipped; justified/finalized come from store post-states.
- Release profile uses `panic = "unwind"` so a verifier panic on a hostile
  proof is contained and rejected.
- Linux-first: PowerShell scripts and Windows-only docs were removed and CI runs
  on Ubuntu; `fetch-leanspec-fixtures.sh` and `legacy-scan.sh` replace their
  PowerShell versions.

### Fixed

- `SignedAggregatedAttestation` SSZ now follows the spec field order (`data`,
  then the offset to `proof`); peers could not decode the old layout.

### Removed

- Statement-based synthetic aggregate proofs (`test-aggregate`), the leanVM IPC
  stubs, `ethean-leanvm-mock`, and the `leansig-backend` / `leanvm-backend`
  features.

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
- Grafana "no data" / epoch-1970 panel issues on long-run dashboards tied to
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

[Unreleased]: https://github.com/ethean-labs/ethean/compare/v0.1.53...HEAD
[0.1.53]: https://github.com/ethean-labs/ethean/compare/v0.1.47...v0.1.53
[0.1.47]: https://github.com/ethean-labs/ethean/compare/v0.1.27...v0.1.47
[0.1.27]: https://github.com/ethean-labs/ethean/compare/v0.1.12...v0.1.27
[0.1.12]: https://github.com/ethean-labs/ethean/releases/tag/v0.1.12
