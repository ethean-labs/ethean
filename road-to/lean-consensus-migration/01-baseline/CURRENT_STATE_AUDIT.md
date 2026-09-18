# Current-State Audit

## Scope and planning boundary

This document audits the **Ethean / Panro Beacon-era tree** at planning baseline commit `880982f9635e8507e5cac37131c3696f6d06191b`. It records what the repository actually implements today and why that surface cannot define Lean Consensus protocol behavior.

**Planning-only boundary:** this baseline task must not modify product code under `src/`, root `Cargo.toml`, binaries, or any executable path. Findings here inform migration phases; they are not permission to patch legacy modules in place.

Evidence sources:

- local baseline audit: `bazalinacaklar/peer-client-library/ethean-current-baseline.md` (2026-09-19);
- tracked path inventory and readable working-tree files at `880982f`;
- peer differential snapshots at fixed commits (implementation evidence only, not protocol authority).

## Peer reference commits

These commits bound the cross-client comparison used in this audit:

| Client | Commit |
| --- | --- |
| Ream | `b003b250f51c038cd5e16b8da02694ee0db1997e` |
| Zeam | `6495beb6b1a584e41c12b3569d9a509abd906259` |
| Qlean-mini | `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0` |
| ethlambda | `313b22d4aa15174b87319002d813926ab7eb6411` |
| Lantern | `440e34727ab3fb447de68d91a1e49be5327706e8` |
| gean | `b78f6d737f4df57a72d5e230635681235fda8024` |
| Peam | `6628e7a564098e592a49b9af0ad7b5dcda0a71fc` |

Normative protocol selection remains a Phase 00 gate. Peer code proves what interoperable clients currently do; it does not override a pinned leanSpec revision.

## Classification policy for this audit

Every protocol-facing surface in the current tree is classified **replace**, not **retain**:

| Label | Meaning in this audit |
| --- | --- |
| **No retain (protocol)** | Serialized types, roots, crypto, consensus rules, wire formats, API constants, and storage records must not survive into the Lean client. Temporary presence during parallel build is comparison input only. |
| **Redesign** | The responsibility (e.g. fork choice, gossip) remains, but the model or wire contract is incompatible with leanSpec-shaped Lean surfaces. |
| **Delete** | Beacon-era or placeholder code with no Lean replacement at the same path. |
| **Generic idea only** | Non-protocol patterns (trait boundaries, rate limits, HTTP composition) may inform **new** crates; they do not authorize keeping old files. See [REMOVAL_REWRITE_REUSE_MAP.md](./REMOVAL_REWRITE_REUSE_MAP.md). |

There is **no retain option** for protocol code in the migration inventory.

## Executive finding

Ethean is a broad **Beacon-era prototype** with Lean/Beam terminology layered on top. It is **not** an interoperable Lean Consensus client.

The seven peers converge on a materially different common surface:

- bounded SSZ containers and SSZ `hash_tree_root`;
- slot-based modified 3SF state with `latest_justified`, `latest_finalized`, historical roots, justified-slot bits, and flattened justification participant bits;
- validator registry entries with **separate attestation and proposal XMSS public keys**;
- raw XMSS / leanSig signatures plus single-message and multi-message leanMultisig proofs;
- distinct block, attestation-subnet, and aggregation gossip topics using SSZ plus raw Snappy;
- Lean fixture runners for SSZ, state transition, fork choice, signatures, networking codecs, and sync.

Ethean does not implement those surfaces. Incremental preservation of current active modules would carry incompatible roots, types, timing, storage, network, API, and identity assumptions into the new client.

## Repository shape at baseline

Tracked implementation surfaces at `880982f` include:

| Surface | Count / status |
| --- | --- |
| Root manifest | `Cargo.toml`, `Cargo.lock`, `README.md`, `.gitignore` |
| Monolithic source | ~93 Rust files under `src/` plus module README files |
| Empty / stray package | `crates/panro-types/` manifest with empty tracked sources (if present at revision) |
| Documentation | ~40 files under `docs/` |
| Examples | `examples/integration_example.rs`, `examples/README.md` (working tree may show deletions) |
| Top-level `config/` | **absent** — config split across `src/config/` and `src/network/network_config.rs` |
| Top-level `tests/` | **absent** — tests live inline in `src/*.rs` |
| Top-level `benches/` | **absent** — benchmarks under `src/bench/`, `src/bin/benchmark.rs`, `src/storage/benchmark.rs`, `src/optimization/` |
| Deployment artifacts | **absent** — prose only in `docs/deployment.md` |

Tests primarily live inline in production modules. That coupling means most “tests” lock placeholder behavior rather than leanSpec conformance.

## Identity and packaging

**Classification: redesign (rename-by-replacement); no retain.**

| Evidence | Finding |
| --- | --- |
| `Cargo.toml` | Package name, default binary, and repository URL still say `panro`; BLS dependencies enabled |
| `src/lib.rs`, `src/client.rs` | Export `PanroClient` and Beacon-named types |
| `src/main.rs`, `src/bin/main.rs` | Advertise “Panro Ethereum Beacon Chain Client” |
| `src/network/network_config.rs` | Emits `panro/1.0.0` protocol identity |
| `src/optimization/Cargo.toml` | Nested manifest inside `src/`; not a workspace member in root manifest |

Peers use explicit Lean package boundaries (e.g. Ream `consensus/lean`, `chain/lean`; other roots name `qlean-mini`, `ethlambda`, `lantern`, `gean`, `peam`). Distinct client names are fine; **Panro identity must not survive** Phase 01.

## Types, serialization, and roots

**Classification: delete current encodings; no retain for protocol types.**

| Evidence | Finding |
| --- | --- |
| `src/types/block.rs` | `BeaconBlock*` types use Serde; block/header/body roots are SHA-256 over `serde_json::to_vec` |
| `src/types/attestation.rs` | Unbounded `Vec<bool>`, `Vec<u8>` for aggregation bits and signatures |
| `src/types/state.rs` | Hand-written state with validators and balances; incomplete vs references elsewhere |
| `src/types/validator.rs` | Single 48-byte-shaped public key, withdrawal credentials, effective balance, slashing, activation/exit epochs |
| `src/storage/database.rs` | Persists typed values as JSON |

Peers derive SSZ encode/decode and `hash_tree_root` on bounded containers (Ream `block.rs`, ethlambda `block.rs`, Lantern `ssz.c`, gean generated tags, Peam `state.rs`, Zeam/Qlean bounded lists).

**Critical:** consensus identity depends on JSON serialization format. Parent validation in `src/consensus/state_transition.rs` and message IDs in `src/network/message_handler.rs` build on non-interoperable roots.

## Cryptography

**Classification: delete BLS, pseudo-Poseidon, and local WOTS; no retain.**

| Evidence | Finding |
| --- | --- |
| `Cargo.toml` | Depends on `blst`, `blstrs`, `bls12_381` |
| `src/crypto/bls.rs` | BLS aggregation and verification |
| `src/consensus/attestation_processing.rs`, `finality.rs`, `slashing.rs` | Embed BLS verification flows |
| `src/bench/mod.rs`, `src/bin/benchmark.rs` | Benchmark BLS |
| `src/crypto/hash.rs` | SHA-256 with `POSEIDON` prefix and simplified rounds — not pinned Poseidon/Poseidon2 |
| `src/crypto/wots/*` | Locally parameterized WOTS+; not leanSig/XMSS library bindings |

Peers bind to leanSig/XMSS with fixed serialized forms and lifetime rules (Ream `post_quantum`, Zeam `hashsig.zig`, Qlean `xmss_provider_impl.cpp`, ethlambda `signature.rs`, Lantern `xmss.c`, gean `xmss/ffi.go`, Peam `crypto/pq.rs`).

Peer snapshots also show **incompatible** PQ parameter choices across clients. Phase 00 must pin one generation before any crypto code is written.

## Consensus, finality, and fork choice

**Classification: delete Beacon/Casper/Tendermint-shaped rules; redesign around pinned Lean algorithm; no retain.**

| Evidence | Finding |
| --- | --- |
| `src/consensus/state_transition.rs` | 4-second slot language with 32-slot epochs, 32 ETH max balance, Beacon activation/withdrawal delays, Casper-FFG-style justification; threshold `2 * total_balance >= 3 * total_balance / 2` is effectively always true for ordinary balances |
| `src/consensus/finality.rs` | Prevote/precommit rounds with fixed 32-unit validator weight; `RealBLSAggregator` |
| `src/consensus/fork_choice.rs` | Labeled LMD-GHOST; maps justified epochs with `epoch * 32` |
| `src/consensus/attestation_processing.rs` | Hardcodes `slot / 32` and 32-slot committees; mock BLS keys |
| `src/consensus/validator_management.rs` | Beacon activation/exit economics |
| `src/consensus/slashing.rs` | Beacon/BLS slashing semantics |
| Multiple modules | Placeholder signature acceptance, mock keys, simplified roots |

Peers implement slot-based modified 3SF transitions and latest-vote fork-choice stores (Ream `state.rs` + `fork_choice/lean`, Zeam `mini_3sf.zig`, Qlean STF, ethlambda transition crate, Lantern `state.c`, gean `finality.go`, Peam `fork_choice.rs`).

## Aggregation

**Classification: redesign; no retain.**

Ethean’s substantive aggregation is BLS point addition in `src/crypto/bls.rs` and committee-index caching in `src/consensus/attestation_processing.rs`. `src/api/validator.rs` returns placeholder aggregated attestations. There is no bounded participant-plus-proof object, recursive child-proof merge, or block-level multi-message proof.

Peers expose explicit single-message and multi-message proof containers in block types and separate aggregation gossip paths.

## Network, gossip, and sync

**Classification: redesign; no retain for wire behavior.**

| Evidence | Finding |
| --- | --- |
| `Cargo.toml` | libp2p TCP/DNS/Noise/Yamux/Gossipsub — no QUIC or req/resp profile matching Lean peers |
| `src/network/mod.rs` | `NetworkManager` starts no transport |
| `src/network/gossip.rs` | In-memory subscription/cache/queue; estimates size instead of SSZ serialization |
| `src/network/peer_manager.rs` | Simulates connection establishment |
| `src/network/protocol.rs` | Returns mock block bytes |
| `src/network/network_config.rs` | Legacy `{network}/beacon_block` topics and Ethereum mainnet TCP bootnodes |
| `src/network/security.rs` | TCP plus WebSocket-shaped transport |
| `src/integration/sync_coordinator.rs` | Generic job orchestration, not Lean status/root/range/checkpoint sync |

Peers use Lean block / attestation-subnet / aggregation topics, SSZ plus raw Snappy, bounded decompression (ethlambda, Zeam, Lantern), QUIC and req/resp where enabled (Peam, Qlean/Ream crates).

## Storage and durability

**Classification: redesign; no retain for persistence format.**

| Evidence | Finding |
| --- | --- |
| `src/storage/database.rs` | `RocksDbBackend` is a no-op: `get` returns `None`, writes succeed without persistence, no flush |
| `src/client.rs` | Uses in-memory database — no restart recovery |
| `src/storage/checkpoints.rs` | Writes zero state-root placeholder |
| Typed records | JSON throughout storage layer |

Peers implement Lean table layouts, checkpoint sync, and durable block/state stores (Ream `storage/tables/lean`, ethlambda `store.rs`, Peam `node/sync`, etc.).

## API, configuration, and operations

**Classification: delete Beacon API constants as protocol truth; generic HTTP ideas may be reimplemented only in new crates.**

| Evidence | Finding |
| --- | --- |
| `src/api/config.rs` | 12-second slots, 32-slot epochs, sync committee constants, 32 ETH max, chain ID 1, mainnet deposit contract |
| `src/api/validator.rs` | Beacon duty/committee/sync-committee routes; placeholder block/aggregate/liveness |
| `src/api/websocket.rs` | Sleeps 12 seconds to simulate block production |
| `src/config/mod.rs` vs `src/network/network_config.rs` | Two inconsistent config systems |
| `src/optimization/*` | ML optimizer, intelligent cache, monitoring dashboard — pre-protocol correctness claims |

Lean config is small and protocol-carried inside SSZ state in peers; node-local config remains separate.

## Tests and conformance

**Classification: delete tests that lock placeholders; no retain.**

| Evidence | Finding |
| --- | --- |
| No top-level `tests/` | No leanSpec fixture runner |
| `src/types/block.rs` tests | Assert JSON-derived hash is 32 bytes |
| `src/main.rs` | Includes `2 + 2 == 4` smoke test |
| `src/storage/database.rs` tests | Exercise in-memory backend while production-named backend discards writes |
| `src/network/gossip.rs` tests | Local queue behavior only |

Peers run Lean fixtures (Ream `lean-spec-tests`, Zeam spectest runner, Qlean vectors, ethlambda blockchain tests, Lantern integration, gean `ssz_test.go`, Peam `lean_spec_fixtures.rs`).

A green legacy test suite is **not** a migration gate.

## Documentation and examples

Root `README.md` and many `docs/` files contain duplicated historical plans, unsupported performance claims, Panro/Beam/Beacon identity, and commands that do not match the active binary. Some newer research docs (`docs/lean-peer-client-research-library-2026-09-19.md`) accurately warn about Lean requirements; they still do not grant retain status to legacy code.

Tracked `examples/integration_example.rs` demonstrates legacy integration patterns. Examples must be replaced or deleted in later phases; none may document nonexistent Lean APIs during migration.

## Critical risks carried into planning

1. **JSON roots** — consensus identity is not SSZ `hash_tree_root`.
2. **Unbounded vectors** — no SSZ maxima on attestations, bits, signatures, or state lists (DoS boundary failure).
3. **BLS embedded in validation** — conflicts with dual XMSS keys and leanMultisig proofs.
4. **Beacon epoch/economics** — internally inconsistent timing (4s vs 12s slots across modules).
5. **Mock network** — no SSZ/Snappy wire path, Lean topics, subnets, QUIC, or req/resp.
6. **No-op RocksDB** — silent data loss on successful write paths.
7. **Placeholder consensus paths** — non-empty signature checks, mock keys, zero checkpoint roots.
8. **Peer pin mismatch** — multiple incompatible crypto and spec generations across the seven snapshots.
9. **Proof and slot budgets** — recursive proving and 4s slots require explicit Phase 00 performance gates.
10. **False conformance** — compiling without fixture runners proves nothing about interop.

## Baseline conclusion

The current Ethean tree is a **Beacon-client prototype and infrastructure sketch**, not an early Lean client. The safe migration stance is **full replacement** behind explicit phase gates:

- Phase 00 freezes pins and fixtures;
- Phases 01–11 build leanSpec-shaped crates and delete superseded paths;
- Phases 12–13 cover RPC/observability and release qualification.

Existing files may remain temporarily as **read-only comparison input** while replacements are built. **No protocol module receives retain approval.** Product code must not be edited as part of this baseline planning task; implementation belongs to the numbered migration phases.

## Related baseline documents

- [LEGACY_COMPONENT_MATRIX.md](./LEGACY_COMPONENT_MATRIX.md) — path-level disposition and phase ownership
- [REMOVAL_REWRITE_REUSE_MAP.md](./REMOVAL_REWRITE_REUSE_MAP.md) — allowed redesign vs forbidden copy
- [../phases/README.md](../phases/README.md) — ordered migration phases
