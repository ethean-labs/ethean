# Phase 03: Canonical SSZ and Types

## Pinned inputs

- Phase snapshot: `LC-D5-2026-09-19`; Ethean planning baseline `880982f9635e8507e5cac37131c3696f6d06191b`.
- `leanSpec` candidate commits: `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54` or `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8`. Either commit is a **candidate only** until Phase 00 verifies repository identity, tree hash, fixture compatibility, and records the accepted value in `spec/pins/phase-03.lock.toml`.
- Accepted `spec/pins/phase-03.lock.toml` produced from Phase 00; no floating branch or unverified constant.
- `MAX_ATTESTATION_DATA` and every container bound must be resolved in Phase 00 with citation and fixture proof. If the value remains open (for example 8 vs 16 unique attestation-data values), **this phase is not ready** and must not start.
- Exact SSZ algorithm version or in-tree implementation decision recorded in the phase lock.
- Peer evidence (differential only): Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, Qlean-mini `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, Gean `b78f6d737f4df57a72d5e230635681235fda8024`, Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.
- Tracked evidence: `docs/lean-peer-client-research-library-2026-09-19.md`. Local peer library notes under `bazalinacaklar/` supplement Phase 00; they are not protocol authority.

## Objective

Replace hand-written Beacon/Serde consensus containers and JSON object-root hashing with bounded Lean containers, canonical SSZ encode/decode, and SSZ hash-tree roots. Delete the entire legacy type surface.

## Non-goals

- No transition semantics, proposer selection, signature verification, genesis policy, fork choice, or finality.
- JSON may exist only in test tooling or non-consensus APIs; it cannot define roots or persisted consensus bytes.
- No aliases named `BeaconBlock`, `BeaconState`, or legacy `Attestation`.
- No production dependency on peer-client codec crates.

## Entry criteria

- Phases 00–02 pass; Phase 02 primitives/profile crates are the only numeric and parameter authorities.
- Phase 00 resolved `MAX_ATTESTATION_DATA`, signature/public-key/proof byte widths, and all container bounds for the selected generation.
- Fixture hashes and schema source locations are present in `spec/pins/phase-03.lock.toml`.
- Any unresolved bound or field order needed by phases 04–06 is a **blocker**, not a TODO default.

## Exact old and new paths

Replace and then delete baseline types:

- `src/types/mod.rs`, `block.rs`, `state.rs`, `validator.rs`, `attestation.rs`, `checkpoint.rs`, `execution.rs`, `README.md`.
- Post–Phase 02 location (must also be deleted): `crates/ethean-node/src/types/` with the same file names if still present.

Create SSZ crate per [TARGET_WORKSPACE](../03-architecture/TARGET_WORKSPACE.md) layering:

- `crates/ethean-ssz/Cargo.toml`, `README.md`, `src/lib.rs`, `src/encode.rs`, `src/decode.rs`, `src/tree_hash.rs`, `src/error.rs`.

Create types crate (workspace member `crates/types`, package `ethean-types`):

- `crates/types/Cargo.toml`, `README.md`, `src/lib.rs`, `src/block/{mod.rs,header.rs,body.rs,signed.rs}`, `src/state/{mod.rs,chain_state.rs,validator.rs,checkpoint.rs}`, `src/operation/{mod.rs,attestation.rs,aggregate.rs,slashing.rs}`, `src/signing/{mod.rs,message.rs,domain.rs}`, `src/receipt/{mod.rs,import.rs,duty.rs}`, `src/error.rs`.

Update consumers (temporary until later phases split them):

- `crates/ethean-node/src/storage/{state.rs,blocks.rs,checkpoints.rs,mod.rs}`, `api/types.rs`, `api/beacon.rs`, and surviving `consensus/*.rs`.

New fixtures/tests:

- `spec/fixtures/phase-03/ssz/`, `spec/fixtures/phase-03/manifest.toml`, `crates/ethean-ssz/tests/`, `crates/types/tests/`, `tests/interop/ssz.rs`.

Every hand-written source file is at most **300 lines**; split by responsibility before review.

## Ordered tasks

1. Transcribe each selected leanSpec schema into bounded Rust containers, preserving exact field order and integer widths.
2. Implement canonical fixed/variable SSZ encoding and strict decoding with checked offsets, lengths, counts, and allocation limits.
3. Implement merkleization, zero hashes, mix-in-length, and container/list roots using pinned SSZ rules.
4. Implement `hash_tree_root` for every consensus type; remove every JSON/SHA-256 object-root helper (including `serde_json::to_vec` hashing in legacy `src/types/block.rs`).
5. Represent signatures, public keys, and proofs with exact bounded byte/container types from the selected generation; verification remains out of scope.
6. Replace storage bytes and roots with SSZ; add a schema/network marker so prototype JSON records are rejected.
7. Update node consumers to Lean names; eliminate Beacon aliases and unbounded `Vec` fields where the pinned schema is bounded.
8. Vendor positive and malformed fixtures with provenance; build round-trip, root, limit-at-`MAX_ATTESTATION_DATA`, and differential tests.
9. Run fuzz/property tests for decoder termination, bounded allocation, and encode/decode/root invariants.
10. Prove deletion: no `src/types/`, no `crates/ethean-node/src/types/`, and no JSON-root path remains reachable from production crates.

## Deletion obligations

- Delete the entire old `src/types/` tree and any `crates/ethean-node/src/types/` forwarding copies.
- Delete `serde_json::to_vec` consensus hashing and manual attestation signing serialization used for roots.
- Delete unbounded `Vec` fields where the pinned schema is bounded.
- Delete legacy execution-payload fields if absent from the pinned Lean schema; do not retain optional compatibility fields.
- Remove Serde derives from consensus types unless a non-consensus adapter has an explicit reviewed need.
- Repository scans may mention deleted paths only in migration/deletion records.

## Security/spec risks

- Offset arithmetic, length mix-ins, field order, or list bounds can create consensus splits.
- Unbounded decode allocations enable denial of service.
- Accepting non-canonical offsets or trailing bytes creates malleability.
- Wrong `MAX_ATTESTATION_DATA` poisons block-body bounds, gossip limits, and proof workloads for all later phases.
- Signature/proof byte sizes must come from the matched cryptography generation, not old BLS/WOTS assumptions.

## Positive and negative fixtures

- Positive: minimum and maximum legal containers, empty/full bounded lists, every upstream root vector, round-trip bytes, known generalized-index proofs, and attestation-data at the resolved `MAX_ATTESTATION_DATA` bound.
- Negative: truncated fixed section, offset before fixed section, descending/overflowing offset, trailing bytes, oversized list (including one element over `MAX_ATTESTATION_DATA`), wrong fixed length, invalid boolean, and mutated root.
- Fixture manifest records URL, upstream commit, SHA-256, and license for every vendored byte.

## Interop and differential tests

- For each pinned leanSpec vector, compare decoded fields, re-encoded bytes, and root exactly.
- Compare roots with two same-generation peer clients when executable; mismatch blocks exit and is triaged against leanSpec, not majority-voted.
- Differential corpus is test-only; no peer codec enters production dependencies.

## Validation commands

```powershell
cargo test -p ethean-ssz -p ethean-types --locked
cargo test --test ssz --locked
cargo fuzz run ssz_decode -- -max_total_time=120
cargo clippy -p ethean-ssz -p ethean-types --all-targets --locked -- -D warnings
rg -n "Beacon(Block|State)|serde_json::to_vec|src/types/|crates/ethean-node/src/types" crates bin tests
rg -n "mod types|pub mod types" crates/ethean-node/src
git diff --check
```

If `cargo-fuzz` is not in the Phase 00 toolchain lock, install the exact locked version before running it.

## Exit criteria

- Every selected Lean consensus type has exact SSZ bytes and roots matching pinned vectors.
- Strict decoders reject the complete negative corpus without excessive allocation or panic.
- Old types and JSON-root code are deleted and unreachable from production crates.
- `MAX_ATTESTATION_DATA` is cited in the phase lock and exercised by at least one positive and one negative fixture.
- Storage and node consumers compile against one Lean type crate; every touched source file is at most 300 lines.

## Rollback/data policy

- Revert the complete phase; no codec feature flag.
- Prototype JSON records are rejected, never auto-converted.
- Test migration utilities may read old fixtures only under `tests/`; no production database migrator is introduced.

## Artifacts/evidence

- Vendored SSZ fixtures and manifest, schema-to-Rust mapping, differential reports, fuzz corpus digest, and `docs/lean-consensus-migration-phase-03-ssz-types.md`.
- Local note under `bazalinacaklar/` records any discrepancy between leanSpec and peer schemas.

## Dependencies

- Requires Phases 00–02.
- Supplies canonical types and roots to Phases 04–06; Phase 07 consumes signing message containers; Phases 08–13 consume wire and storage encodings derived here.
