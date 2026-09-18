# Phase 05: State Transition

## Pinned inputs

- Phase snapshot: `LC-D5-2026-09-19`; Ethean planning baseline `880982f9635e8507e5cac37131c3696f6d06191b`.
- `leanSpec` candidate commits: `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54` or `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8`. Either commit is a **candidate only** until Phase 00 verifies and records the accepted value in `spec/pins/phase-05.lock.toml`.
- Accepted `spec/pins/phase-05.lock.toml` with exact transition functions, processing order, validity predicates, proposer/attester data rules, historical-root behavior, and transition fixtures.
- Canonical types/roots from Phase 03 and profile/genesis/clock semantics from Phase 04.
- Signature/proof verification is enabled only if Phase 00 pinned the matching implementation and parameter set; otherwise signed inputs are rejected as unsupported rather than accepted with placeholders.
- Peer evidence (differential only): Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, Qlean-mini `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, Gean `b78f6d737f4df57a72d5e230635681235fda8024`, Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.
- Tracked evidence: `docs/lean-peer-client-research-library-2026-09-19.md`. Local notes under `bazalinacaklar/` supplement Phase 00; unresolved transition-order or bound values are **blockers**.

## Objective

Replace the Beacon-era transition, committee, reward, slashing, and block-processing pipeline with a pure, transactional, fixture-driven Lean state transition. Produce deterministic post-state roots and typed failures for every conformance vector.

## Non-goals

- No fork-choice head selection or finality-store mutation; those belong to Phase 06.
- No BLS, GRANDPA, Casper FFG, Beacon committee/reward lifecycle, prevote/precommit rounds, or execution-payload compatibility.
- No fake-signature acceptance in production builds.
- No I/O, wall clock, network, or global mutable singletons inside the transition crate.

## Entry criteria

- Phases 03–04 pass all canonical type, genesis, profile, and clock tests.
- Phase 00 resolved transition order, all bounds (including attestation-data limits from Phase 03), signature/proof behavior, and fixture digests.
- Every old transition responsibility has a delete/replace mapping in the compatibility ledger.

## Exact old and new paths

Replace and then delete baseline consensus transition files:

- `src/consensus/state_transition.rs`, `block_processing.rs`, `attestation_processing.rs`, `validator_management.rs`, `slashing.rs` (and post–Phase 02 copies under `crates/ethean-node/src/consensus/`).

Create transition crate per [TARGET_WORKSPACE](../03-architecture/TARGET_WORKSPACE.md):

- `crates/ethean-transition/Cargo.toml`, `README.md`, `src/lib.rs`, `src/context.rs`, `src/outcome.rs`, `src/error.rs`.
- `src/block/{mod.rs,validate.rs,apply.rs}`, `src/operation/{mod.rs,attestation.rs,aggregate.rs,slashing.rs}`, `src/slot/{mod.rs,process.rs}`.
- Include `src/epoch/` modules **only** if the pinned Lean generation defines epoch processing; otherwise omit and prove no Beacon epoch path remains.

Update integration/storage (temporary):

- `crates/ethean-node/src/storage/{state.rs,blocks.rs,checkpoints.rs,mod.rs}`, `integration/coordinator.rs`, `integration/sync_coordinator.rs`, `client.rs`.

Retain only pending Phase 06 files in `crates/ethean-node/src/consensus/`:

- `fork_choice.rs`, `finality.rs`, `performance.rs`, `mod.rs`, `README.md` until Phase 06 deletes them.

New fixtures/tests:

- `spec/fixtures/phase-05/transition/`, `spec/fixtures/phase-05/manifest.toml`, `crates/ethean-transition/tests/`, `tests/interop/transition.rs`.

Every hand-written source file is at most **300 lines**; split by responsibility before review.

## Ordered tasks

1. Translate the pinned transition call graph into small pure functions preserving normative order and pre/postconditions.
2. Implement slot processing, skipped-slot handling, historical-root updates, block-header/body processing, and attestation/justification participation exactly as pinned.
3. Implement validator/index selection and any balance/participation changes only when present in the selected Lean generation.
4. Verify parent, proposer, state, body, attestation, and signature/proof inputs at their normative boundary.
5. Make transition execution atomic: apply to an owned/copy-on-write state and publish/store only after all checks and final root succeed.
6. Return structured deterministic errors; never panic on peer-controlled bytes or indices.
7. Replace node storage/integration calls with the new crate; persist canonical SSZ bytes plus verified roots under the Lean schema.
8. Run every positive and negative transition fixture; add property tests for determinism and failure atomicity.
9. Remove old dependencies and prove no BLS/Beacon/Casper/GRANDPA transition symbols remain.
10. Document any leanSpec ambiguity resolved during implementation in phase evidence; do not encode peer-specific branches in production.

## Deletion obligations

- Delete all five old transition files listed above and their exports/tests; no wrappers or compatibility shims.
- Delete JSON state-root calculation, modulo proposer selection tied to Beacon epochs, genesis-always-valid handling, empty-signature checks, Beacon reward/penalty logic, committee shuffling, and execution-payload hashing.
- Delete `RealBLSAggregator` from transition dependencies and all production mock key/signature generation.
- Delete any transition feature that can bypass verification in a production build.
- Delete Casper-FFG-style justification thresholds and prevote/precommit state machines from `src/consensus/finality.rs` interaction paths (full removal completes in Phase 06).

## Security/spec risks

- Processing-order differences and partial mutation on failure cause consensus splits or corrupted state.
- Skipped-slot and historical-root off-by-one errors can poison later roots and fork-choice inputs.
- Unchecked validator indices, list growth, or arithmetic can panic or exhaust resources.
- Retaining Beacon epoch math (`slot / 32`) silently desynchronizes from Lean 4-second slot semantics.
- Stateful XMSS/proof verification boundaries must match the pinned generation; an unavailable verifier is a hard unsupported-input error.

## Positive and negative fixtures

- Positive: genesis-to-first-block, skipped slots, empty permitted collections, maximum permitted attestation data at the Phase 03 bound, valid participation updates, and multi-block root chains.
- Negative: wrong slot/parent/proposer/state root, unknown validator, duplicate/out-of-range participant, invalid source/target relation, oversized list, malformed signature/proof, and post-state-root mismatch.
- Atomicity fixtures assert the input state and database remain unchanged for each rejected operation.

## Interop and differential tests

- Run all pinned leanSpec pre-state/input/post-state vectors and compare post-state SSZ/root exactly.
- Differentially execute a deterministic multi-slot trace in at least two same-generation clients.
- On divergence, minimize the trace and resolve against the pinned transition function; never add a peer-specific production branch.

## Validation commands

```powershell
cargo test -p ethean-transition --locked
cargo test --test transition --locked
cargo test --workspace transition_determinism --locked
cargo test --workspace transition_atomicity --locked
cargo clippy -p ethean-transition --all-targets --locked -- -D warnings
rg -n "Beacon|BLS|RealBLSAggregator|serde_json|GRANDPA|Casper|prevote|precommit|execution_payload|slot as usize.*%" crates/ethean-transition crates/ethean-node/src/consensus
git diff --check
```

Forbidden search hits must be zero except explicit negative-test names.

## Exit criteria

- All pinned positive post-state roots match and all negative fixtures reject with no mutation.
- Old transition/attestation/validator/slashing/block-processing files are deleted.
- Production transition has no fake verifier, BLS, JSON root, or Beacon fallback.
- Storage publishes only successfully verified Lean states/blocks.
- Transition crate is I/O-free and every touched source file is at most 300 lines.

## Rollback/data policy

- Revert the complete phase; do not retain a transition selector.
- Phase-05 data uses a new schema version and cannot be opened by the prototype.
- Failed transitions leave no partial records; there is no automatic conversion of old states or blocks.

## Artifacts/evidence

- Transition fixture manifest, conformance report, minimized divergence traces, atomicity report, forbidden-symbol audit, and `docs/lean-consensus-migration-phase-05-state-transition.md`.
- Local research note under `bazalinacaklar/` captures any leanSpec ambiguity resolved during implementation.

## Dependencies

- Requires Phases 00–04.
- Supplies validated states, blocks, attestations, and roots to Phase 06; Phases 07–13 consume transition outcomes through the node pipeline.
