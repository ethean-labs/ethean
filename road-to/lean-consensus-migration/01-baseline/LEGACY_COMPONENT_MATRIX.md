# Legacy Component Matrix

## Purpose

This matrix inventories every tracked product surface at planning baseline `880982f9635e8507e5cac37131c3696f6d06191b`. Each row assigns a **disposition**, **owning phase (00–13)**, **replacement path**, and **notes**.

There is **no retain disposition**. Generic engineering ideas do not keep old paths alive; see [REMOVAL_REWRITE_REUSE_MAP.md](./REMOVAL_REWRITE_REUSE_MAP.md).

## Disposition legend

| Disposition | Meaning |
| --- | --- |
| **replace-then-delete** | Build the Lean replacement first; delete the old path when the phase exit gate passes |
| **rename-by-replacement** | New Ethean identity/path supersedes the old; old identity-bearing artifacts are removed |
| **delete** | Remove with no replacement at the same path |

## Phase legend

| Phase | Scope |
| --- | --- |
| 00 | Research, pins, fixture policy |
| 01 | Identity, repository cleanup |
| 02 | Workspace, primitives, profile |
| 03 | Canonical SSZ and types |
| 04 | Genesis presets and clock |
| 05 | State transition |
| 06 | Fork choice and finality |
| 07 | XMSS signer safety |
| 08 | leanVM aggregation |
| 09 | Validator and node duties |
| 10 | QUIC, gossip, req/resp |
| 11 | Storage, sync, checkpoints |
| 12 | RPC, metrics, observability |
| 13 | Retirement, soak, release qualification |

Replacement paths follow [../03-architecture/TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md).

---

## Root surfaces

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `Cargo.toml` | rename-by-replacement | 01 → 02 | Root virtual workspace `Cargo.toml` | Package `panro`, BLS deps, monolithic `[package]` become workspace root in Phase 02 |
| `Cargo.lock` | replace-then-delete | 02 | New workspace `Cargo.lock` | Regenerated after workspace split; old lock deleted at Phase 02 exit |
| `README.md` | replace-then-delete | 01 → 13 | Root `README.md` (Ethean index) | Strip false claims incrementally in 01; final rewrite after executable Lean path in 13 |
| `.gitignore` | replace-then-delete | 02 | Root `.gitignore` | Add workspace/build paths; remove obsolete monolith entries |
| `crates/panro-types/` (if tracked) | delete | 01 | — | Empty stray manifest; superseded by `crates/types/` |
| `LICENSE` (if tracked) | rename-by-replacement | 01 | `LICENSE` | Metadata only; no protocol coupling |

---

## `src/` — entry and library shell

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/lib.rs` | replace-then-delete | 01 → 02 | Removed with monolith | Exports `PanroClient`, Beacon types |
| `src/main.rs` | replace-then-delete | 01 → 02 | `bin/ethean/src/main.rs` | Duplicate/historical entry; Beacon network names |
| `src/client.rs` | replace-then-delete | 01 → 02 | `crates/node/src/service/` | `PanroClient` orchestration |
| `src/cli.rs` | replace-then-delete | 01 → 02 | `bin/ethean/src/cli.rs` | Historical CLI flags |
| `src/README.md` | replace-then-delete | 01 | `crates/*/README.md`, `bin/ethean/README.md` | Monolith layout doc |
| `src/bin/main.rs` | replace-then-delete | 01 → 02 | `bin/ethean/src/main.rs` | Default binary `panro` |
| `src/bin/benchmark.rs` | delete | 01 | `benches/` (Lean benches) | BLS benchmark binary |
| `src/bin/README.md` | replace-then-delete | 01 | `bin/ethean/README.md` | |

---

## `src/types/` — JSON / Serde consensus roots

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/types/mod.rs` | replace-then-delete | 03 | `crates/types/src/lib.rs` | |
| `src/types/block.rs` | replace-then-delete | 03 | `crates/types/src/block/` | **JSON SHA-256 roots**, not SSZ |
| `src/types/attestation.rs` | replace-then-delete | 03 | `crates/types/src/operation/attestation.rs` | **Unbounded `Vec`** |
| `src/types/state.rs` | replace-then-delete | 03 | `crates/types/src/state/` | Beacon balances model |
| `src/types/validator.rs` | replace-then-delete | 03 | `crates/types/src/state/validator.rs` | Single BLS-shaped key |
| `src/types/checkpoint.rs` | replace-then-delete | 03 | `crates/types/src/state/checkpoint.rs` | Beacon checkpoint semantics |
| `src/types/execution.rs` | delete | 03 | — | Execution payload assumptions not in Lean scope |
| `src/types/README.md` | replace-then-delete | 03 | `crates/types/README.md` | |

---

## `src/consensus/` — Beacon rules and placeholders

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/consensus/mod.rs` | replace-then-delete | 05–06 | `crates/transition/`, `crates/fork-choice/` | |
| `src/consensus/state_transition.rs` | replace-then-delete | 05 | `crates/transition/src/` | Epoch/balance/Casper-FFG; broken threshold |
| `src/consensus/fork_choice.rs` | replace-then-delete | 06 | `crates/fork-choice/src/` | LMD-GHOST; `epoch * 32` |
| `src/consensus/finality.rs` | replace-then-delete | 06 | `crates/fork-choice/src/update/` | Prevote/precommit; **BLS** |
| `src/consensus/attestation_processing.rs` | replace-then-delete | 05 | `crates/transition/src/operation/attestation.rs` | Mock BLS; 32-slot committees |
| `src/consensus/block_processing.rs` | replace-then-delete | 05 | `crates/transition/src/block/` | JSON-root validation |
| `src/consensus/validator_management.rs` | replace-then-delete | 09 | `crates/validator/src/duty/` | **32 ETH** Beacon economics |
| `src/consensus/slashing.rs` | replace-then-delete | 05 | `crates/transition/src/operation/slashing.rs` | **BLS** slashing |
| `src/consensus/performance.rs` | delete | 13 | `crates/metrics/` | Synthetic perf structs |
| `src/consensus/README.md` | replace-then-delete | 05–06 | Crate READMEs | |

---

## `src/crypto/` — BLS, fake Poseidon, local WOTS

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/crypto/mod.rs` | replace-then-delete | 03 → 07 | `crates/crypto/src/lib.rs` | |
| `src/crypto/bls.rs` | delete | 03 | — | **BLS**; no Lean equivalent |
| `src/crypto/hash.rs` | delete | 03 | `crates/crypto/src/hash/` | **Fake Poseidon** prefix on SHA-256 |
| `src/crypto/wots.rs` | delete | 07 | — | Local WOTS entry |
| `src/crypto/wots/mod.rs` (via wots.rs) | delete | 07 | — | |
| `src/crypto/wots/params.rs` | delete | 07 | — | Local parameters |
| `src/crypto/wots/keygen.rs` | delete | 07 | — | |
| `src/crypto/wots/sign.rs` | delete | 07 | — | |
| `src/crypto/wots/verify.rs` | delete | 07 | — | |
| `src/crypto/wots/README.md` | delete | 07 | `crates/crypto/README.md` | |
| `src/crypto/README.md` | replace-then-delete | 07 | `crates/crypto/README.md` | |

---

## `src/network/` — mock network and Beacon topics

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/network/mod.rs` | replace-then-delete | 10 | `crates/network/src/service/` | Empty `NetworkManager` |
| `src/network/gossip.rs` | replace-then-delete | 10 | `crates/network/src/gossip/` | **In-memory mock** gossip |
| `src/network/network_config.rs` | replace-then-delete | 01 → 10 | `crates/network/src/config.rs`, `crates/profile/` | Beacon topics; mainnet bootnodes |
| `src/network/peer_manager.rs` | replace-then-delete | 10 | `crates/network/src/peer/` | Simulated connections |
| `src/network/protocol.rs` | delete | 10 | `crates/network-wire/src/request/` | **Mock block bytes** |
| `src/network/message_handler.rs` | replace-then-delete | 10 | `crates/network-wire/src/gossip/codec.rs` | Simplified roots for IDs |
| `src/network/security.rs` | replace-then-delete | 10 | `crates/network/src/limits/` | TCP/WebSocket only |
| `src/network/discovery.rs` | replace-then-delete | 10 | `crates/network/src/discovery/` | Not Lean discv5/QUIC profile |
| `src/network/connection_manager.rs` | replace-then-delete | 10 | `crates/network/src/peer/manager.rs` | |
| `src/network/bandwidth.rs` | replace-then-delete | 10 | `crates/network/src/limits/rate.rs` | Generic idea → redesign in Lean boundary |
| `src/network/performance.rs` | delete | 13 | `crates/metrics/src/record/network.rs` | |
| `src/network/orchestrator.rs` | replace-then-delete | 10 | `crates/network/src/service/runner.rs` | |
| `src/network/README.md` | replace-then-delete | 10 | `crates/network/README.md`, `crates/network-wire/README.md` | |

---

## `src/storage/` — JSON records and no-op RocksDB

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/storage/mod.rs` | replace-then-delete | 11 | `crates/storage/src/lib.rs` | |
| `src/storage/database.rs` | replace-then-delete | 11 | `crates/storage/src/backend/` | **No-op RocksDB**; JSON values; trait idea only |
| `src/storage/memory_backend.rs` | replace-then-delete | 11 | `crates/storage/src/backend/memory.rs` | Test backend pattern OK in new crate |
| `src/storage/blocks.rs` | replace-then-delete | 11 | `crates/storage/src/record/block.rs` | Legacy record shape |
| `src/storage/state.rs` | replace-then-delete | 11 | `crates/storage/src/record/state.rs` | |
| `src/storage/checkpoints.rs` | replace-then-delete | 11 | `crates/sync/src/planner/checkpoint.rs` | **Zero state-root placeholder** |
| `src/storage/cache.rs` | replace-then-delete | 11 | `crates/storage/src/` (internal cache) | Generic cache idea only |
| `src/storage/index.rs` | replace-then-delete | 11 | `crates/storage/src/key/schema.rs` | |
| `src/storage/migrations.rs` | delete | 11 | `crates/storage/src/migration/` | Beacon JSON schema |
| `src/storage/backup.rs` | replace-then-delete | 11 | Operator docs + storage API | Not protocol |
| `src/storage/benchmark.rs` | delete | 13 | `benches/storage/` | Legacy JSON backend |
| `src/storage/README.md` | replace-then-delete | 11 | `crates/storage/README.md` | |

---

## `src/api/` — Beacon API and placeholders

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/api/mod.rs` | replace-then-delete | 12 | `crates/rpc/src/server/` | |
| `src/api/beacon.rs` | delete | 12 | — | **Beacon API** surface |
| `src/api/validator.rs` | replace-then-delete | 12 | `crates/rpc/src/routes/validator.rs` | Beacon duties; placeholders |
| `src/api/config.rs` | delete | 12 | `crates/rpc/src/routes/node.rs` | **12s slots, 32 ETH**, chain ID 1 |
| `src/api/node.rs` | replace-then-delete | 12 | `crates/rpc/src/routes/node.rs` | |
| `src/api/debug.rs` | replace-then-delete | 12 | `crates/rpc/src/routes/` (debug gated) | |
| `src/api/websocket.rs` | replace-then-delete | 12 | `crates/rpc/src/ws/` | **12s sleep** simulation |
| `src/api/middleware.rs` | replace-then-delete | 12 | `crates/rpc/src/server/` | Axum pattern → new crate only |
| `src/api/error.rs` | replace-then-delete | 12 | `crates/rpc/src/error.rs` | |
| `src/api/types.rs` | replace-then-delete | 12 | `crates/rpc/src/types/` | Beacon DTOs |
| `src/api/README.md` | replace-then-delete | 12 | `crates/rpc/README.md` | |

---

## `src/integration/` — legacy integration layer

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/integration/mod.rs` | delete | 10–11 | `crates/node/src/pipeline/` | **Legacy integration** |
| `src/integration/coordinator.rs` | delete | 11 | `crates/node/src/service/runner.rs` | |
| `src/integration/sync_coordinator.rs` | replace-then-delete | 11 | `crates/sync/src/service/` | Generic jobs, not Lean sync |
| `src/integration/network_storage.rs` | delete | 11 | `crates/node/src/persistence/` | Couples mock net + JSON store |
| `src/integration/bridge.rs` | delete | 10 | — | Ad hoc bridge |
| `src/integration/conflict_resolver.rs` | delete | 11 | — | UUID conflict toy |
| `src/integration/tests.rs` | delete | 13 | `tests/interop/` | Inline integration tests |
| `src/integration/README.md` | delete | 11 | `crates/sync/README.md` | |

---

## `src/optimization/` — ML optimizer and pre-Lean claims

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/optimization/mod.rs` | delete | 13 | — | **ML optimizer** module tree |
| `src/optimization/ml_optimizer.rs` | delete | 13 | — | **ML optimizer** |
| `src/optimization/intelligent_cache.rs` | delete | 13 | — | Unverified cache ML |
| `src/optimization/monitoring_dashboard.rs` | replace-then-delete | 12 | `crates/metrics/`, observability docs | Dashboard claims without exporter |
| `src/optimization/integration.rs` | delete | 13 | — | |
| `src/optimization/Cargo.toml` | delete | 01 | — | Stray nested manifest |
| `src/optimization/README.md` | delete | 13 | `road-to/lean-consensus-migration/06-observability/` | |

---

## `src/config/`, `src/utils/`, `src/bench/`

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/config/mod.rs` | replace-then-delete | 01 → 04 | `bin/ethean/src/config/`, `crates/profile/` | Split from network_config |
| `src/config/README.md` | replace-then-delete | 04 | `crates/profile/README.md` | |
| `src/utils/mod.rs` | delete | 02 | `crates/primitives/src/` | Move only if still needed after audit |
| `src/utils/README.md` | delete | 02 | — | |
| `src/bench/mod.rs` | delete | 13 | `benches/` | **BLS** microbenches |
| `src/bench/README.md` | delete | 13 | `benches/README.md` | |

---

## `docs/` — historical and misleading documentation

All docs describe the Beacon-era monolith unless noted. Disposition is **replace-then-delete** for current-architecture claims or **delete** for superseded weekly noise. Final Ethean docs are written against executable Lean crates in Phases 12–13.

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `docs/README.md` | replace-then-delete | 13 | `docs/README.md` | Index rewrite |
| `docs/architecture.md` | replace-then-delete | 13 | `docs/architecture.md` | Monolith architecture |
| `docs/consensus.md` | delete | 06 | `docs/consensus.md` (new) | Beacon consensus description |
| `docs/networking.md` | replace-then-delete | 10 | `docs/networking.md` | Mock/libp2p TCP picture |
| `docs/storage.md` | replace-then-delete | 11 | `docs/storage.md` | JSON / no-op RocksDB |
| `docs/security.md` | replace-then-delete | 07 | `docs/security.md` | BLS/WOTS claims |
| `docs/deployment.md` | replace-then-delete | 13 | `docs/deployment.md` | **Deployment** prose only; no artifacts |
| `docs/monitoring.md` | replace-then-delete | 12 | `docs/monitoring.md` | No Prometheus exporter today |
| `docs/roadmap.md` | delete | 13 | `road-to/lean-consensus-migration/` | Superseded by migration plan |
| `docs/TODO_IMPLEMENT.md` | delete | 13 | — | Stale task list |
| `docs/development-notes.md` | delete | 13 | — | Historical |
| `docs/connection-management-plan.md` | delete | 10 | — | Beacon networking plan |
| `docs/english.md` | replace-then-delete | 01 | — | Meta; keep short note if needed |
| `docs/folder-readmes-and-local-conventions.md` | replace-then-delete | 02 | `docs/contributing.md` | |
| `docs/source-file-size-limit.md` | replace-then-delete | 02 | `.cursor/rules` + `docs/contributing.md` | Policy doc |
| `docs/peer-reference-clients.md` | replace-then-delete | 00 | `docs/peer-reference-clients.md` | Research index |
| `docs/lean-peer-client-research-library-2026-09-19.md` | replace-then-delete | 00 | Updated research summary | Accurate Lean gap analysis — still not retain for code |
| `docs/leanroadmap-local-notes.md` | replace-then-delete | 00 | `docs/leanroadmap-local-notes.md` | |
| `docs/week5-planning-database-integration.md` | delete | 13 | — | Weekly report |
| `docs/WEEK5_DEVELOPMENT_NOTES.md` | delete | 13 | — | Weekly report |
| `docs/week6-phase2-completion.md` | delete | 13 | — | |
| `docs/week7-database-optimization-completion.md` | delete | 13 | — | |
| `docs/week8-phase2-development.md` | delete | 13 | — | |
| `docs/week8-final-summary.md` | delete | 13 | — | |
| `docs/week8-p2p-networking-phase1-start.md` | delete | 13 | — | |
| `docs/week8-p2p-networking-phase1-complete.md` | delete | 13 | — | |
| `docs/week9-security-performance-development.md` | delete | 13 | — | |
| `docs/week-10-database-integration.md` | delete | 13 | — | |
| `docs/week10-database-integration-notes.md` | delete | 13 | — | |
| `docs/week10-planning-database-integration.md` | delete | 13 | — | |
| `docs/week-11-advanced-optimization.md` | delete | 13 | — | **ML optimization** claims |
| `docs/week11-advanced-optimization.md` | delete | 13 | — | Duplicate weekly |
| `docs/week-12-security-cryptography.md` | delete | 13 | — | BLS/WOTS era |
| `docs/development/week5-plan.md` | delete | 13 | — | |
| `docs/development/phase2-sprint2.1-week1-state-transition-complete.md` | delete | 13 | — | Beacon sprint |
| `docs/development/phase2-sprint2.1-week2-validator-management-plan.md` | delete | 13 | — | |
| `docs/development/phase2-sprint2.1-week2-validator-management-complete.md` | delete | 13 | — | |
| `docs/development/phase2-sprint2.1-week3-attestation-processing-plan.md` | delete | 13 | — | |
| `docs/development/phase2-sprint2.1-week3-attestation-processing-advanced.md` | delete | 13 | — | |
| `docs/development/phase2-sprint2.1-week4-finalization-plan.md` | delete | 13 | — | |
| `docs/development/phase2-sprint2.1-complete-status.md` | delete | 13 | — | |

---

## `examples/`

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `examples/README.md` | replace-then-delete | 12 | `examples/README.md` | |
| `examples/integration_example.rs` | delete | 12 | `examples/node_minimal.rs` (new) | **Legacy integration** example |

---

## `config/` (top-level)

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| *(missing)* | replace-then-delete | 04 | `bin/ethean/` config samples | No tracked top-level config dir today |

---

## `tests/` (top-level)

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| *(missing)* | replace-then-delete | 03+ | `tests/interop/`, crate `tests/` | Inline `src/**/*.rs` tests delete with owning modules |
| Inline tests in `src/**/*.rs` (~50 files) | delete | Owner phase | Crate/unit tests + `tests/interop/` | Shape/placeholder tests must not survive |

---

## `benches/` (top-level)

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| *(missing)* | replace-then-delete | 13 | `benches/` | Replace `src/bench/`, `src/bin/benchmark.rs`, `src/storage/benchmark.rs` |
| `src/bench/mod.rs` | delete | 13 | `benches/crypto`, `benches/sync` | BLS benches |
| `src/bin/benchmark.rs` | delete | 13 | `benches/` | |
| `src/storage/benchmark.rs` | delete | 13 | `benches/storage/` | JSON backend |

---

## `deployment/` (top-level)

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| *(missing)* | replace-then-delete | 13 | `.github/`, `deploy/` (when introduced) | Today only `docs/deployment.md` prose |
| `docs/deployment.md` | replace-then-delete | 13 | `docs/deployment.md` + future manifests | No Docker/K8s/terraform at baseline |

---

## Monolith retirement (entire tree)

| Path / surface | Disposition | Phase | Replacement path | Notes |
| --- | --- | --- | --- | --- |
| `src/` (entire directory) | delete | 02 → 13 | `crates/*`, `bin/ethean/` | Deleted after last responsibility migrates; see [../05-retirement/REQUIRED_DIRECTORY_POLICY.md](../05-retirement/REQUIRED_DIRECTORY_POLICY.md) |

---

## Cross-cutting markers (must not survive)

These patterns appear across multiple rows above and are always **replace-then-delete** or **delete**:

| Marker | Example paths | Phase | Notes |
| --- | --- | --- | --- |
| JSON consensus roots | `src/types/block.rs`, `src/consensus/*`, `src/network/message_handler.rs` | 03 | SSZ `hash_tree_root` replaces entirely |
| Serde unbounded types | `src/types/attestation.rs`, `src/types/state.rs` | 03 | Bounded SSZ lists |
| BLS | `src/crypto/bls.rs`, `src/consensus/finality.rs`, benches | 03–07 | Removed from dependency graph |
| Local WOTS | `src/crypto/wots/*` | 07 | leanSig/XMSS backend |
| Beacon epoch/economics | `src/consensus/state_transition.rs`, `src/api/config.rs` | 04–05 | Slot-based Lean profile |
| Mock network | `src/network/gossip.rs`, `protocol.rs`, `peer_manager.rs` | 10 | QUIC + SSZ/Snappy |
| No-op RocksDB | `src/storage/database.rs` | 11 | Real backend + SSZ records |
| Beacon API | `src/api/beacon.rs`, `validator.rs`, `config.rs` | 12 | Lean RPC subset |
| Legacy integration | `src/integration/*`, `examples/integration_example.rs` | 10–11 | Node pipeline + sync |
| ML optimizer | `src/optimization/ml_optimizer.rs`, related docs | 13 | Delete outright |

---

## Verification queries

Phase owners should be able to answer “is this path gone?” with repository searches:

```powershell
# Forbidden identity
rg -i "panro|PanroClient" --glob "!road-to/**"

# Forbidden crypto
rg "blst|blstrs|bls12_381" Cargo.toml crates/

# Forbidden roots
rg "serde_json::to_vec" crates/

# Forbidden API constants
rg "SLOT.*12|EPOCH.*32|deposit.contract" crates/
```

Exit gate: **zero matches** in production crates after the owning phase completes.

## Related documents

- [CURRENT_STATE_AUDIT.md](./CURRENT_STATE_AUDIT.md)
- [REMOVAL_REWRITE_REUSE_MAP.md](./REMOVAL_REWRITE_REUSE_MAP.md)
- [../phases/README.md](../phases/README.md)
