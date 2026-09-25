# Phase 08 — leanVM Aggregation

## Pinned inputs

- Snapshot `LC-D5-2026-09-19`, Ethean baseline `880982f`, `leanSpec` `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54`, `leanSig` `15cbdd43ec8525aa43fea2f42cafc5ed366084ae`, and `leanVM` `e2592df4e30fdddbbf8ae26a333116c68cec7026`.
- Type-1 means one message plus explicit unique participants; Type-2 means the ordered proposal component and selected Type-1 attestation components merged into one block proof. Proof bytes are bounded at 524,288.
- Profile constants: `LOG_INV_RATE=2`, 52-byte XMSS keys, 2,536-byte raw signatures, eight unique attestation-data values per block, 4,096 validators, 4-second slots, and five intervals.
- Rust `1.97.1`, lockfile, target triple, CPU baseline, native libraries, Rayon limit, and builder image digest are fixed in the Phase 00 ledger.
- Differential peers: Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, and gean `b78f6d737f4df57a72d5e230635681235fda8024`.

## Objective

Implement bounded Type-1 and Type-2 proof verification and production with explicit public-input binding, deterministic participant handling, isolated native proving, and deadlines that cannot block chain progress.

## Non-goals

- No STF proof, generic zkVM framework, proof-market protocol, alternate hash line, or devnet-2 `leanMultisig` compatibility.
- No fake proof accepted by production code and no decoding heuristic between proof generations.
- No promise that every valid signature is included; selection policy is deterministic and measured.

## Entry criteria

- Phase 07 signer and Phase 06 canonical roots/fork choice are complete.
- Phase 03 containers encode raw, Type-1, and Type-2 objects without ambiguity and reject over-limit data before allocation.
- The pinned fixture manifest includes signature/proof vectors or records their exact absence and a generated conformance corpus.
- CPU, memory, thread, queue, and slot-deadline budgets are approved for the reference host and cgroup.

## Exact old and new paths

Replace and delete aggregation behavior in `src/crypto/bls.rs`, `src/consensus/attestation_processing.rs`, `src/api/validator.rs`, `src/integration/`, and BLS benches in `src/bench/`.

Create:

- `crates/ethean-crypto/src/aggregation/{mod.rs,bindings.rs,statement.rs,verify.rs,prove.rs}`.
- `crates/ethean-node/src/aggregation/{mod.rs,pool.rs,selection.rs,worker.rs,budget.rs,recovery.rs}`.
- `crates/ethean-types/src/proofs/{mod.rs,type1.rs,type2.rs,participants.rs}`.
- `tests/fixtures/aggregation/`, `tests/interop/aggregation.rs`, `tests/negative/proof_inputs.rs`, and `benches/aggregation/`.

Each new code directory receives an English `README.md`; every hand-written source file remains at most 2000 lines.

## Ordered tasks

1. Bind the exact leanVM revision and expose a narrow safe Rust API; assert ABI, parameter fingerprint, proof generation, target, and feature set at startup.
2. Define canonical statements containing protocol profile, proof type, message root, slot, ordered public keys, ordered unique participant indices, and child-proof descriptors.
3. Reject empty/duplicate/out-of-range participants, wrong bitlist lengths, overlap, message mismatch, noncanonical order, excess recursion, oversized proof, and unknown generation before FFI.
4. Re-derive all public inputs from parent state and block contents. Wire metadata never overrides consensus-derived messages, keys, slots, or participant sets.
5. Verify Type-1 and Type-2 proofs behind bounded semaphores after cheap SSZ, topology, lifetime, and participant checks.
6. Key the pool by profile plus canonical attestation-data root; retain raw signatures, valid child proofs, coverage, age, and source/target relevance with bounded variants and TTL.
7. Merge only same-message disjoint coverage. Validate every produced proof locally before pool insertion or publication; a native success code is insufficient.
8. Build Type-2 components in canonical block order, with the proposer Type-1 component in the pinned position. Re-derive and verify the final proof before signing/publishing the block envelope.
9. Run proving in a dedicated process with memory, CPU, thread, wall-time, and output-size limits. Kill and replace a wedged worker; never execute native proving on the chain owner.
10. Implement admission deadlines, cooperative cancellation between calls, stale-parent checks before and after proving, proposal priority, and discard metrics for late results.
11. Make proof caches revision-, profile-, statement-, and binary-fingerprint-specific; never cache failed verification as valid.
12. Benchmark flat, recursive, Type-2 merge, split, and verification shapes. Set budgets from p50/p95/p99 on the reference host, including cgroup-constrained runs.

## Legacy deletion obligations

- Remove BLS point addition, placeholder aggregate API responses, committee-index caches shaped around BLS, and old aggregation benchmarks.
- Remove all concatenated-signature, opaque-proof, devnet-2, and fake-production paths.
- New and old aggregation cannot coexist as selectable production modes.

## Security and specification risks

- A proof may verify against attacker-selected inputs unless every input is re-derived.
- Native FFI may crash, corrupt memory, ignore cancellation, oversubscribe threads, or wedge indefinitely.
- Overlapping participants can double-count votes; noncanonical ordering can fork proof statements.
- `LOG_INV_RATE`, hash parameters, and proof generation are security/interop inputs, not tuning knobs.
- Proof decompression, pool variants, splitting, and invalid-proof floods are CPU/RAM denial-of-service surfaces.
- A valid but late proof can cause stale-parent proposals or reduce finality coverage.

## Tests

- Fixtures: raw-to-Type-1, child merge, Type-1 verify, ordered Type-1-to-Type-2 merge, Type-2 verify/split, minimum/maximum participants, and exact serialization.
- Negative: wrong root/slot/key/profile/type, duplicate/overlapping/out-of-order participants, invalid child, altered component order, truncated/oversized proof, unsupported parameter, and valid proof with mismatched block contents.
- Recovery: kill/timeout/OOM/panic the prover at every boundary; restart worker, preserve validated pool data, discard partial output, and keep chain owner responsive.
- Interop: bidirectional Type-1/Type-2 production and verification with pinned peers; compare participant coverage, public inputs, and block proof ordering.
- Performance: sustained worst-valid verification, invalid-proof flood, cgroup CPU/memory pressure, queue saturation, and a 4-second-slot deadline simulation.

## Validation commands

```text
cargo +1.97.1 test -p ethean-crypto aggregation --release --locked
cargo +1.97.1 test -p ethean-node aggregation --release --locked
cargo +1.97.1 test --test aggregation --release --locked
cargo +1.97.1 test --test proof_inputs --release --locked
cargo +1.97.1 bench --bench aggregation --locked
cargo +1.97.1 clippy -p ethean-crypto -p ethean-node --all-targets --locked -- -D warnings
```

## Exit criteria

- Every accepted proof is bound to consensus-derived inputs and the pinned generation.
- Produced proofs are self-verified; 100,000 mutation cases yield no false acceptance.
- Prover crash, OOM, and wedge do not stall slot ticks, networking, storage, or shutdown.
- Reference-host p99 budgets fit the approved interval deadline and publish a measured capacity envelope.
- Legacy BLS/placeholder aggregation is deleted and all touched source files are at most 2000 lines.

## Rollback and data policy

Proof pools and caches are disposable; raw signed duties and signer journals are not. A rollback invalidates cached proofs whenever binary, profile, leanVM, or statement fingerprints differ. Persisted blocks remain immutable and are reverified before import. Partial prover output is never resumed or trusted.

## Evidence artifacts

- ABI/parameter fingerprint, fixture manifest, differential matrix, mutation corpus results, resource profiles, deadline traces, cgroup report, worker fault matrix, and deterministic selection transcript under `artifacts/phase-08/`.

## Dependencies

Depends on Phases 03, 05, 06, and 07. Phase 09 schedules production; Phase 10 gossips proofs; Phase 11 persists bounded pools; Phases 12–13 monitor and qualify the complete path.
