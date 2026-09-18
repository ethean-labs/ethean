# Removal, Rewrite, and Reuse Map

## Purpose

This document separates **conceptual responsibilities** from **legacy implementations**. It defines what may be reimplemented test-first inside new Lean crates versus what must be **deleted without rewrite**, and it forbids copying old source into the new tree.

Planning baseline: `880982f9635e8507e5cac37131c3696f6d06191b`.

**Hard rule:** no production file under the current `src/` tree is copied, moved, or “adapted” into `crates/`. Migration is replacement behind tests and fixtures, not a rename pass.

## Decision framework

```text
Legacy module touched?
        │
        ├─ Encodes protocol bytes, roots, crypto, consensus rules, wire topics,
        │  API constants, or storage record layout?
        │     └─ DELETE without rewrite. Rebuild from leanSpec + pinned fixtures.
        │
        ├─ Generic infrastructure (trait, limiter, HTTP middleware, metric name)?
        │     └─ REDESIGN inside Lean crate boundaries only; do not port file.
        │
        └─ Placeholder / mock / ML / Beacon-only economics?
              └─ DELETE. No Lean counterpart at same abstraction.
```

| Outcome | Meaning |
| --- | --- |
| **Delete without rewrite** | Remove code and tests; do not create a line-by-line port |
| **Redesign (test-first)** | New public API in a named target crate; behavior proved by fixtures and interop tests |
| **Forbidden** | Copy-paste, `mod legacy`, thin wrapper over old struct, or `#[path = "../src/..."]` |

There is **no reuse disposition** for protocol implementations.

---

## Delete without rewrite

These responsibilities are wrong at the specification level. They must not reappear under new names.

### Consensus and types

| Legacy locus | Why delete | Peer / spec evidence |
| --- | --- | --- |
| JSON SHA-256 roots (`src/types/block.rs`) | Not SSZ `hash_tree_root`; not canonical | All seven peers use SSZ tree hashing |
| Hand-written Serde containers with unbounded `Vec` | No DoS bounds; not wire-compatible | Ream/ethlambda/Lantern bounded containers |
| Beacon block/header/body split as implemented | Field layout and roots differ from Lean | leanSpec Lean containers |
| Casper-FFG / epoch balance justification | Wrong finality model | Modified 3SF slot logic in peers |
| LMD-GHOST store (`src/consensus/fork_choice.rs`) | Wrong fork-choice algorithm | Latest-vote Lean stores |
| Prevote/precommit rounds (`src/consensus/finality.rs`) | Tendermint-shaped, BLS-backed | Slot-based 3SF in peers |
| 32 ETH effective balance defaults | Beacon economics | Lean registry without Beacon balance field |
| Activation/exit epoch lifecycle | Beacon validator lifecycle | Compact Lean validator index + dual keys |
| Execution payload types (`src/types/execution.rs`) | Out of Lean consensus scope | Execution coupling deferred |
| Placeholder signature checks | Unsafe if reachable | Explicit XMSS verify only |

### Cryptography

| Legacy locus | Why delete |
| --- | --- |
| `src/crypto/bls.rs` and BLS in consensus | Lean uses XMSS / leanSig, not BLS |
| `blst`, `blstrs`, `bls12_381` dependencies | Must leave dependency graph |
| `src/crypto/hash.rs` “Poseidon” prefix | Not a pinned Poseidon/Poseidon2 implementation |
| Entire `src/crypto/wots/*` tree | Local WOTS params ≠ leanSig pin |
| BLS benchmarks (`src/bench/`, `src/bin/benchmark.rs`) | Misleading performance signal |

### Network and wire

| Legacy locus | Why delete |
| --- | --- |
| In-memory gossip queue (`src/network/gossip.rs`) | Not a network service |
| Mock protocol responses (`src/network/protocol.rs`) | Not a codec |
| Simulated peer connections (`src/network/peer_manager.rs`) | Hides real failure modes |
| Beacon topic names and mainnet bootnodes | Wrong protocol namespace |
| Empty `NetworkManager` (`src/network/mod.rs`) | No transport lifecycle |
| Message IDs from simplified roots | Must use spec hash and topic rules |

### Storage

| Legacy locus | Why delete |
| --- | --- |
| No-op `RocksDbBackend` | Silent data loss |
| JSON value encoding for chain data | Not SSZ typed records |
| Zero checkpoint state root placeholder | Breaks sync trust model |
| Beacon-oriented migrations | Wrong schema version stream |

### API and product claims

| Legacy locus | Why delete |
| --- | --- |
| Beacon REST constants (`src/api/config.rs`) | Wrong slot/epoch/deposit truth |
| Beacon validator duty routes as implemented | Wrong duty model |
| 12-second WebSocket simulation | Wrong slot clock |
| ML optimizer module (`src/optimization/ml_optimizer.rs`) | No protocol basis |
| Intelligent cache / dashboard ML claims | Unverified; distracts from correctness |

### Integration and examples

| Legacy locus | Why delete |
| --- | --- |
| `src/integration/*` coordinator/bridge/conflict | Ad hoc coupling of mock layers |
| `examples/integration_example.rs` | Documents non-Lean wiring |

### Tests to discard

| Pattern | Why delete |
| --- | --- |
| JSON root length == 32 bytes | Proves wrong hash property |
| `2 + 2 == 4` in `src/main.rs` | Non-gate |
| In-memory DB tests while RocksDB no-op | False confidence |
| Gossip queue ordering tests | Local mock only |

---

## Redesign test-first (allowed conceptual reuse)

These areas may **inform** new designs. Implementation must be written fresh inside the target crate, driven by pinned fixtures and public API tests. Reading old code for behavioral comparison is allowed during migration; **copying is not**.

### Storage backend trait and batching

| Idea from legacy | Target | Constraints |
| --- | --- | --- |
| `DatabaseBackend` trait, in-memory test double | `crates/storage/src/backend/` | New trait signatures; SSZ typed keys/values; atomic batch; real RocksDB or approved backend |
| Cache/index separation | `crates/storage/src/` internals | No JSON records; cache is non-authoritative |
| Backup/export hooks | Operator tooling in Phase 11+ | Must not read legacy JSON |

**Gate tests:** crash injection, restart reconstruction, schema fingerprint mismatch rejection, round-trip SSZ record encode/decode.

### Rate limits, bandwidth, connection accounting

| Idea from legacy | Target | Constraints |
| --- | --- | --- |
| Bandwidth counters, connection limits | `crates/network/src/limits/` | Bound decode size **before** decompression; align with Snappy/SSZ limits from Phase 10 |
| Peer scoring concepts | `crates/network/src/peer/score.rs` | Scores observe behavior; must not replace consensus validation |

**Gate tests:** oversize frame rejection, connection cap enforcement, metric cardinality bounds.

### HTTP server composition (Axum)

| Idea from legacy | Target | Constraints |
| --- | --- | --- |
| Router layout, middleware, error mapping | `crates/rpc/src/server/` | New routes for Lean operator surface only |
| WebSocket fan-out pattern | `crates/rpc/src/ws/` | Subscriptions on Lean head/duties, not 12s fake producer |

**Gate tests:** route contract tests, auth/TLS policy, no Beacon constant leakage in JSON responses.

### Metrics and observability

| Idea from legacy | Target | Constraints |
| --- | --- | --- |
| Counter/histogram recording | `crates/metrics/` | `ethean_` namespace; cardinality caps |
| Monitoring dashboard concept | `road-to/lean-consensus-migration/06-observability/` | Prometheus/Grafana provisioning, not in-process fake stats |

**Gate tests:** descriptor uniqueness, readiness gating, scrape budget from Phase 00 performance ledger.

### Module boundaries (not code)

| Legacy separation | Target crates | Notes |
| --- | --- | --- |
| types / consensus / crypto / network / storage / api | `types`, `crypto`, `transition`, `fork-choice`, `network-wire`, `network`, `storage`, `sync`, `validator`, `node`, `rpc` | Boundaries are reused as **architecture**, not as file moves |
| Client orchestration | `crates/node`, `bin/ethean` | Single owner of mutable chain state |

### Sync orchestration (generic only)

| Idea from legacy | Target | Constraints |
| --- | --- | --- |
| Job queue / retry naming | `crates/sync/src/download/` | Jobs are Lean status/root/range/checkpoint shaped |
| `sync_coordinator` naming | `crates/sync/src/service/` | Must not call legacy integration types |

**Gate tests:** pinned sync fixtures, bad-root rejection, checkpoint trust rules from Phase 11.

### Config loading (non-protocol)

| Idea from legacy | Target | Constraints |
| --- | --- | --- |
| File/env configuration | `bin/ethean/src/config/` | Separated from SSZ `ChainProfile` in `crates/profile/` |
| Network presets | `crates/profile/src/preset.rs` | Values from Phase 04 pins, not `src/api/config.rs` |

---

## Explicit forbiddens

The following are **never** acceptable migration shortcuts:

1. **Copying old source** into `crates/` with edits (including “temporary” copies).
2. **Thin wrappers** that delegate to `src/consensus`, `src/types`, or `src/crypto`.
3. **Feature flags** that select legacy vs Lean production paths.
4. **Serde compatibility layers** that accept old JSON records in production.
5. **Dual binaries** (`panro` + `ethean`) after Phase 01 exit.
6. **Adapters** on hot paths that translate JSON roots to SSZ at runtime.
7. **Importing peer client code** as templates (peer-reference rule).
8. **Retaining BLS** for “transition period” verification.
9. **Keeping inline legacy tests** because they pass.
10. **ML / optimization modules** repackaged as performance hooks without Phase 00 budgets.

If a developer needs to compare behavior, use:

- pinned leanSpec fixtures in `spec/fixtures/`;
- differential tests under `tests/interop/`;
- read-only inspection of legacy files scheduled for deletion.

Do not add new dependencies from new crates to old `src/` modules.

---

## Responsibility → target crate map

| Responsibility | Legacy (delete) | New home | First proving phase |
| --- | --- | --- | --- |
| Hash32, Slot, indices | `src/types/*` helpers | `crates/primitives` | 02 |
| Chain profile / limits | `src/config/`, `src/network/network_config.rs` | `crates/profile` | 04 |
| SSZ types + roots | `src/types/*` | `crates/types` + SSZ crate | 03 |
| State transition | `src/consensus/state_transition.rs` | `crates/transition` | 05 |
| Fork choice / finality | `src/consensus/fork_choice.rs`, `finality.rs` | `crates/fork-choice` | 06 |
| XMSS verify/sign | `src/crypto/wots/*`, BLS paths | `crates/crypto` | 07 |
| Aggregation proofs | BLS aggregation | `crates/crypto/src/aggregate/` | 08 |
| Validator duties | `src/consensus/validator_management.rs`, `src/api/validator.rs` | `crates/validator` | 09 |
| Gossip + req/resp | `src/network/*` | `crates/network-wire`, `crates/network` | 10 |
| Persistence + sync | `src/storage/*`, `src/integration/sync_coordinator.rs` | `crates/storage`, `crates/sync` | 11 |
| Operator RPC | `src/api/*` | `crates/rpc`, `crates/metrics` | 12 |
| Process wiring | `src/client.rs`, `src/main.rs` | `bin/ethean`, `crates/node` | 02–13 |

---

## Test-first replacement order

For each redesigned responsibility:

1. Import or generate **pinned fixtures** (Phase 00 manifest).
2. Write **public API tests** in the target crate against fixtures.
3. Implement minimal code to pass positive and negative vectors.
4. Add **interop differential** tests in `tests/interop/` where peers agree on the same pin.
5. Wire through `crates/node` behind a single production path.
6. Delete the legacy file(s) listed in [LEGACY_COMPONENT_MATRIX.md](./LEGACY_COMPONENT_MATRIX.md).
7. Run repository search gates (identity, crypto, JSON roots) for the phase.

Skipping step 6 is a phase exit failure even if the new crate passes tests.

---

## Documentation and planning artifacts

| Artifact class | Treatment |
| --- | --- |
| `road-to/lean-consensus-migration/**` | Preserved as migration authority; updated by planning tasks |
| `docs/week*` and sprint reports | Delete at Phase 13; never source-of-truth for implementation |
| `docs/lean-peer-client-research-library-*.md` | Update after Phase 00 refresh; does not authorize code retain |
| Root README | Rewrite incrementally; must not claim Lean interop before Phase 10+ gates |

---

## Summary stance

| Category | Action |
| --- | --- |
| Protocol bytes, roots, crypto, consensus, wire, API constants, storage layout | **Delete without rewrite** |
| Generic traits, limits, HTTP shape, metrics | **Redesign test-first** in named Lean crates |
| Placeholders, mocks, ML optimizer, Beacon economics | **Delete** |
| Old source copy into new tree | **Forbidden** |

The legacy tree is comparison input until deletion. The Lean client is a new implementation graph validated by pins, fixtures, and phase gates—not a refactor of Panro.

## Related documents

- [CURRENT_STATE_AUDIT.md](./CURRENT_STATE_AUDIT.md)
- [LEGACY_COMPONENT_MATRIX.md](./LEGACY_COMPONENT_MATRIX.md)
- [../03-architecture/TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md)
- [../05-retirement/REQUIRED_DIRECTORY_POLICY.md](../05-retirement/REQUIRED_DIRECTORY_POLICY.md)
