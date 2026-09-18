# Deletion Register

This register inventories every major baseline path that must disappear or be reclassified during the Lean Consensus migration. A row closes only when its replacement is active, its prerequisite is proven, the baseline path is deleted, and the verification search returns no unexpected hits.

Deletion phases (R1–R5) align with migration work:

| Phase | Scope |
| --- | --- |
| **R1** | Workspace foundation: manifests, primitives, profile, types, executable skeleton |
| **R2** | Pure core: cryptography, transition, fork choice |
| **R3** | Adapters: storage, wire protocol, network, RPC, metrics |
| **R4** | Orchestration: sync, validator, node ownership, executable wiring, examples |
| **R5** | Retirement: documentation classification, legacy-name purge, obsolete-directory removal |

**No empty legacy directories may remain.** After each batch delete, run the empty-directory scan in [REQUIRED_DIRECTORY_POLICY.md](./REQUIRED_DIRECTORY_POLICY.md). Required top-level dirs are recreated with an English `README.md`, not `.gitkeep`.

---

## Root package identity (`panro`)

| Baseline path | Replacement owner | Deletion prerequisite | New path | Verification |
| --- | --- | --- | --- | --- |
| `Cargo.toml` — `name = "panro"`, `default-run = "panro"`, `[[bin]] name = "panro"`, BLS deps, old repository URL | R1 | Virtual workspace root lists only target members; `cargo metadata` succeeds; binary starts from new package | Root virtual `Cargo.toml`; `bin/ethean/Cargo.toml`; `crates/*/Cargo.toml` | `rg -n '^\[package\]|^default-run|^name\s*=\s*"panro"|blst|blstrs|bls12_381' Cargo.toml` |
| `Cargo.lock` (panro-era resolution graph) | R1 | Regenerated from workspace after dependency purge | Workspace `Cargo.lock` pinned to Lean profile | `rg -n 'panro' Cargo.lock` (expect none) |
| `crates/panro-types/` (empty scaffold) | R1 | Target `crates/types/` (or `crates/ethean-types/`) active with SSZ types | `crates/types/` per [TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md) | `rg -n 'panro-types|crates/panro' Cargo.toml crates` |
| `src/lib.rs` — `PanroClient`, legacy exports | R4 | External imports use target crate APIs only | `crates/*/src/lib.rs` facades | `rg -n 'PanroClient|panro::|crate::panro' crates bin tests examples` |

---

## Loose root source files

| Baseline path | Replacement | Prerequisite | New path | Verification |
| --- | --- | --- | --- | --- |
| `src/main.rs` | R4 | CLI smoke tests pass | `bin/ethean/src/main.rs` | `rg -n '^src/main\.rs' Cargo.toml` |
| `src/client.rs` | R4 | Node service owns mutable chain state | `crates/node/src/service/` | `rg -n 'src/client\.rs|mod client|crate::client' .` |
| `src/cli.rs` | R4 | CLI snapshot tests pass | `bin/ethean/src/cli.rs` | `rg -n 'src/cli\.rs|mod cli' crates bin` |
| `src/config/mod.rs` | R1/R4 | Profile validation separated from process config | `crates/profile/`, `bin/ethean/src/config/` | `rg -n 'src/config|crate::config' crates bin tests` |

After closure: delete `src/config/` including its README. Do not leave an empty shell.

---

## `src/types/` — Beacon hand-written containers

**Baseline files:**

```text
src/types/mod.rs
src/types/attestation.rs
src/types/block.rs
src/types/checkpoint.rs
src/types/execution.rs
src/types/state.rs
src/types/validator.rs
src/types/README.md
```

| Field | Value |
| --- | --- |
| **Replacement owner** | R1 — `crates/primitives/`, `crates/profile/`, `crates/types/src/{block,state,operation,signing,receipt}/` |
| **Prerequisite** | Canonical SSZ serialization, bounded collections, hash-tree roots, transition fixture tests; no JSON/Serde consensus encoding |
| **New path** | See [phases/03-canonical-ssz-and-types.md](../phases/03-canonical-ssz-and-types.md) |
| **Verification** | `rg -n 'src/types|crate::types|panro::types|BeaconBlock|BeaconState' crates bin tests examples benches docs road-to --glob '!lean-consensus-migration/05-retirement/**'` |

**Delete:** all listed files, then `src/types/`.

---

## `src/crypto/` — BLS, pseudo-Poseidon, local WOTS

**Baseline files:**

```text
src/crypto/mod.rs
src/crypto/hash.rs
src/crypto/bls.rs
src/crypto/wots.rs
src/crypto/wots/params.rs
src/crypto/wots/keygen.rs
src/crypto/wots/sign.rs
src/crypto/wots/verify.rs
src/crypto/README.md
src/crypto/wots/README.md
```

| Field | Value |
| --- | --- |
| **Replacement owner** | R2 — `crates/crypto/src/{hash,signature,aggregate}/` with parameters from `ethean-profile` |
| **Prerequisite** | Known-answer vectors, malformed-input rejection, domain separation, signature and aggregate-proof tests; no BLS or local WOTS API exported |
| **New path** | [phases/07-xmss-signer-safety.md](../phases/07-xmss-signer-safety.md), [phases/08-leanvm-aggregation.md](../phases/08-leanvm-aggregation.md) |
| **Verification** | `rg -n -i 'src/crypto|crate::crypto|\bbls(t|12|trs)?\b|\bwots\b|blst|blstrs' crates bin tests examples benches docs --glob '!lean-consensus-migration/05-retirement/**'` |

**Delete:** all listed files, then `src/crypto/wots/` and `src/crypto/`.

---

## `src/consensus/` — Beacon-era transition and fork choice

**Baseline files:**

```text
src/consensus/mod.rs
src/consensus/attestation_processing.rs
src/consensus/block_processing.rs
src/consensus/finality.rs
src/consensus/fork_choice.rs
src/consensus/performance.rs
src/consensus/slashing.rs
src/consensus/state_transition.rs
src/consensus/validator_management.rs
src/consensus/README.md
```

| Field | Value |
| --- | --- |
| **Replacement owner** | R2/R4 — `crates/transition/`, `crates/fork-choice/`, `crates/node/src/pipeline/`, `crates/validator/` |
| **Prerequisite** | State-transition vectors, fork-choice scenarios, finality cases, invalid-operation tests; pure core has no I/O |
| **New path** | [phases/05-state-transition.md](../phases/05-state-transition.md), fork-choice phase |
| **Verification** | `rg -n 'src/consensus|crate::consensus|Casper|epoch \* 32|GRANDPA' crates bin tests examples docs --glob '!lean-consensus-migration/05-retirement/**'` |

**Delete:** all listed files, then `src/consensus/`.

---

## `src/storage/` — JSON records and no-op RocksDB

**Baseline files:**

```text
src/storage/mod.rs
src/storage/backup.rs
src/storage/benchmark.rs
src/storage/blocks.rs
src/storage/cache.rs
src/storage/checkpoints.rs
src/storage/database.rs
src/storage/index.rs
src/storage/memory_backend.rs
src/storage/migrations.rs
src/storage/state.rs
src/storage/README.md
```

| Field | Value |
| --- | --- |
| **Replacement owner** | R3 — `crates/storage/src/{api,record,key,backend,migration}/` |
| **Prerequisite** | Atomic batch, restart recovery, schema migration, corruption detection, backend parity tests |
| **New path** | [phases/11-storage-sync-and-checkpoints.md](../phases/11-storage-sync-and-checkpoints.md) |
| **Verification** | `rg -n 'src/storage|crate::storage|serde_json.*state|database\.rs' crates bin tests examples docs --glob '!lean-consensus-migration/05-retirement/**'` |

**Delete:** all listed files, then `src/storage/`.

---

## `src/network/` — simulated Beacon gossip

**Baseline files:**

```text
src/network/mod.rs
src/network/bandwidth.rs
src/network/connection_manager.rs
src/network/discovery.rs
src/network/gossip.rs
src/network/message_handler.rs
src/network/network_config.rs
src/network/orchestrator.rs
src/network/peer_manager.rs
src/network/performance.rs
src/network/protocol.rs
src/network/security.rs
src/network/README.md
```

| Field | Value |
| --- | --- |
| **Replacement owner** | R3 — `crates/network-wire/`, `crates/network/` |
| **Prerequisite** | Bounded decode, topic/version match, malformed message, peer lifecycle, req/resp, local interop tests |
| **New path** | [phases/10-quic-gossip-and-reqresp.md](../phases/10-quic-gossip-and-reqresp.md) |
| **Verification** | `rg -n 'src/network|/eth2/|BeaconBlock|yamux|noise' crates bin tests examples docs --glob '!lean-consensus-migration/05-retirement/**'` |

**Delete:** all listed files, then `src/network/`.

---

## `src/api/` — Beacon REST surface

**Baseline files:**

```text
src/api/mod.rs
src/api/beacon.rs
src/api/config.rs
src/api/debug.rs
src/api/error.rs
src/api/middleware.rs
src/api/node.rs
src/api/types.rs
src/api/validator.rs
src/api/websocket.rs
src/api/README.md
```

| Field | Value |
| --- | --- |
| **Replacement owner** | R3 — `crates/rpc/src/{server,routes,ws,types}/` |
| **Prerequisite** | Route contract, limits, error mapping, cancellation, readiness tests via node handles |
| **New path** | RPC crate per target workspace |
| **Verification** | `rg -n 'src/api|api::beacon|sync_committee|deposit_contract|12.second' crates bin tests examples docs --glob '!lean-consensus-migration/05-retirement/**'` |

**Delete:** all listed files, then `src/api/`.

---

## `src/integration/` and `src/optimization/`

**Baseline files:**

```text
src/integration/mod.rs
src/integration/bridge.rs
src/integration/conflict_resolver.rs
src/integration/coordinator.rs
src/integration/network_storage.rs
src/integration/sync_coordinator.rs
src/integration/tests.rs
src/integration/README.md
src/optimization/mod.rs
src/optimization/integration.rs
src/optimization/intelligent_cache.rs
src/optimization/ml_optimizer.rs
src/optimization/monitoring_dashboard.rs
src/optimization/Cargo.toml
src/optimization/README.md
```

| Field | Value |
| --- | --- |
| **Replacement owner** | R3/R4 — `crates/sync/`, `crates/node/`, `crates/metrics/`; integration tests under `tests/` |
| **Prerequisite** | Orchestration, backpressure, restart, sync, metrics, shutdown tests; no ML bridge in consensus path |
| **New path** | Target orchestration crates |
| **Verification** | `rg -n 'src/(integration|optimization)|ml_optimizer|intelligent_cache|monitoring_dashboard' crates bin tests examples docs --glob '!lean-consensus-migration/05-retirement/**'` |

**Delete:** both directories entirely, including nested manifests.

---

## `src/bin/` and `src/bench/`

| Baseline path | Replacement | Prerequisite | New path | Verification |
| --- | --- | --- | --- | --- |
| `src/bin/main.rs` | R4 | Production binary is `ethean` | `bin/ethean/src/main.rs` | `rg -n 'src/bin/main' Cargo.toml` |
| `src/bin/benchmark.rs` | R4 | Benchmarks use public crate APIs | `crates/*/benches/`, `benches/` | `rg -n 'src/bin/benchmark' Cargo.toml` |
| `src/bin/README.md` | R4 | Documented under `bin/ethean/README.md` | `bin/ethean/README.md` | path absent |
| `src/bench/mod.rs`, `src/bench/README.md` | R4 | Workspace benches run | `benches/README.md` + crate benches | `rg -n 'src/bench' .` |
| `src/utils/mod.rs`, `src/utils/README.md` | R1–R4 | Utilities owned by target crates | per-crate modules | `rg -n 'src/utils|mod utils' crates bin tests` |

**Delete:** `src/bin/`, `src/bench/`, `src/utils/` after row closure.

When every source row above is closed, **delete the top-level `src/` directory**. `Test-Path src` must be `$false`. An empty `src/` is a failure.

---

## Examples

| Baseline path | Replacement | Prerequisite | New path | Verification |
| --- | --- | --- | --- | --- |
| `examples/integration_example.rs` | R4 | Example compiles against public workspace crates and `ethean` binary; no legacy `src::` imports | `examples/<lean-topic>.rs` using `crates/*` public API | `rg -n -i 'panro|beam|bls|wots|src::|crate::panro|integration_example' examples` |
| `examples/README.md` (if it references panro/legacy APIs) | R4 | README documents only current examples | Rewritten `examples/README.md` | `rg -n -i 'panro|legacy|src/' examples/README.md` |

If examples remain required, rewrite in place. Do not keep historical examples “for reference.”

---

## Old `road-to/` planning documents

Each file below is **deleted or reclassified as historical** in R5. Valid requirements must appear in `road-to/lean-consensus-migration/` before deletion.

| Baseline path | Replacement owner | Prerequisite | New path | Verification |
| --- | --- | --- | --- | --- |
| `road-to/Ethean_MANIFESTO.md` | R5 | Charter and scope captured in `00-charter/` | [PROJECT_CHARTER.md](../00-charter/PROJECT_CHARTER.md), [SCOPE_AND_NON_GOALS.md](../00-charter/SCOPE_AND_NON_GOALS.md) | `rg -n 'Ethean_MANIFESTO' README.md docs road-to --glob '!lean-consensus-migration/**'` |
| `road-to/MODULAR_ARCHITECTURE.md` | R5 | Module policy in architecture docs | [DEPENDENCY_RULES.md](../03-architecture/DEPENDENCY_RULES.md), [MODULE_SIZE_POLICY.md](../03-architecture/MODULE_SIZE_POLICY.md), [TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md) | `rg -n 'MODULAR_ARCHITECTURE\.md' README.md docs road-to` |
| `road-to/TECHNOLOGY_DECISION.md` | R5 | Technology choices in protocol/architecture records | `02-protocol/`, `03-architecture/`, phase decision notes | `rg -n 'TECHNOLOGY_DECISION\.md' README.md docs road-to` |
| `road-to/WEEK5_PROGRESS_REPORT.md` | R5 | Historical only — unique metrics extracted to migration evidence | git history or classified archive | `rg -n 'WEEK5_PROGRESS_REPORT' README.md docs road-to` |
| `road-to/WEEK6_TRANSITION_PLAN.md` | R5 | Sequencing superseded by `phases/` | [phases/README.md](../phases/README.md) | `rg -n 'WEEK6_TRANSITION_PLAN' README.md docs road-to` |
| `road-to/phase-1-completion-summary.md` | R5 | No active README links | git history | `rg -n 'phase-1-completion-summary' README.md docs road-to` |
| `road-to/storage-layer-progress-report.md` | R5 | Storage plan in phase 11 | [phases/11-storage-sync-and-checkpoints.md](../phases/11-storage-sync-and-checkpoints.md) | `rg -n 'storage-layer-progress-report' README.md docs road-to` |
| `road-to/panro-roadmap-vision.md` | R5 | Product identity is Ethean Lean Consensus Client | [README.md](../README.md), charter | `rg -n -i 'panro-roadmap-vision' road-to --glob '!lean-consensus-migration/05-retirement/**'` |
| `road-to/panro-todo-list.md` | R5 | Open work in phase register / issue tracker | `phases/`, `04-risks/RISK_REGISTER.md` | `rg -n -i 'panro-todo-list' road-to --glob '!lean-consensus-migration/05-retirement/**'` |
| `road-to/beam-chain-client-master-plan.md` | R5 | Lean migration plan is authoritative | `lean-consensus-migration/` tree | `rg -n -i 'beam-chain-client-master-plan|beam master plan' road-to --glob '!lean-consensus-migration/05-retirement/**'` |
| `road-to/wots-implementation-report.md` | R5 | Crypto migration in phases 07–08 | [phases/07-xmss-signer-safety.md](../phases/07-xmss-signer-safety.md) | `rg -n -i 'wots-implementation-report' road-to docs` |
| `road-to/notlar.txt` | R5 | Valid English requirements extracted; file deleted | Current planning docs only | `Test-Path road-to/notlar.txt` → `$false` |
| `road-to/README.md` (legacy index) | R5 | Migration root index or historical index with clear classification | [lean-consensus-migration/README.md](../README.md) | links resolve; no normative panro/beam guidance |

Historical material may remain only when labeled history and compliant with [LEGACY_NAME_ALLOWLIST.md](./LEGACY_NAME_ALLOWLIST.md).

---

## Old `docs/` classes (summary)

Full class inventory remains in the prior register revision. R5 deletes or archives:

- weekly development/status reports (`docs/WEEK*.md`, sprint plans under `docs/development/`);
- legacy architecture guides (`docs/architecture.md`, `consensus.md`, `networking.md`, etc.);
- generic backlogs (`docs/TODO_IMPLEMENT.md`, `roadmap.md`);
- superseded convention copies now owned by Cursor rules and migration docs.

**Verification:** `rg -n -i 'week[- ]?[0-9]+|panro|beam[ -]?chain|\bbls\b' docs --glob '*.md' --glob '!lean-consensus-migration/**'`

Inspect each hit; allowlisted retirement/baseline paths only.

---

## Empty-directory and final verification

After R5, from repository root:

```powershell
# Obsolete monolithic src must not exist
if (Test-Path src) { throw "src/ still exists" }

# No empty directories anywhere
$empty = Get-ChildItem -Directory -Recurse |
  Where-Object { -not (Get-ChildItem $_.FullName -Force) }
if ($empty) { $empty.FullName; throw "Empty directories prohibited" }

# Workspace health
cargo metadata --all-features --format-version 1
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

# Legacy names on active surface (see LEGACY_NAME_ALLOWLIST.md for exclusions)
rg -n -i 'panro|beam[ -]?chain|\bbls\b|\bwots\b|eth2|beacon' `
  Cargo.toml crates bin tests examples benches docs README.md `
  --glob '*.rs' --glob '*.toml' --glob '*.md' --glob '*.json' --glob '*.yaml'

# Obsolete path references
rg -n 'src/(api|bench|bin|config|consensus|crypto|integration|network|optimization|storage|types|utils)' `
  Cargo.toml crates bin tests examples benches docs README.md `
  --glob '!lean-consensus-migration/05-retirement/**'
```

Legacy-name hits must be zero on the active surface. Obsolete-path hits may appear only in this deletion register and other allowlisted retirement/baseline documents.
