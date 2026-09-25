# Phase 06: Fork Choice and Finality

## Pinned inputs

- Phase snapshot: `LC-D5-2026-09-19`; Ethean planning baseline `880982f9635e8507e5cac37131c3696f6d06191b`.
- `leanSpec` candidate commits: `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54` or `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8`. Either commit is a **candidate only** until Phase 00 verifies and records the accepted value in `spec/pins/phase-06.lock.toml`.
- Accepted `spec/pins/phase-06.lock.toml` naming the **exact finality/fork-choice generation**. Only **modified 3SF-mini** or **PQ heartbeat / Goldfish** may be implemented, and only if Phase 00 pinned that generation with tick, vote-promotion, safe-target, tie-break, justification, and finality fixtures. Implementing both, mixing generations, or defaulting to Beacon Casper/epoch rules is forbidden.
- Canonical types/roots from Phase 03, profile/genesis/clock from Phase 04, and validated transition outputs from Phase 05.
- Peer evidence (differential only): Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, Qlean-mini `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, Gean `b78f6d737f4df57a72d5e230635681235fda8024`, Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.
- Tracked evidence: `docs/lean-peer-client-research-library-2026-09-19.md`. Local notes under `bazalinacaklar/` supplement Phase 00; an unresolved fork-choice generation (OSD-006) is a **blocker**.

## Objective

Implement pure Lean fork-choice store updates, vote promotion, safe-target selection, pruning, and finality from the pinned snapshot. Delete LMD-GHOST, GRANDPA, prevote/precommit, and Beacon epoch-finality behavior from production paths.

## Non-goals

- No networking, gossip validation, storage persistence, validator duty scheduling, or XMSS signing.
- No BLS vote aggregation, Casper FFG epoch checkpoints, or `epoch * 32` head mapping.
- No dual fork-choice engines, feature-flagged legacy head selection, or peer-majority rule selection.
- Goldfish/PQ-heartbeat behavior is out of scope unless explicitly pinned in Phase 00; roadmap direction alone is insufficient.

## Entry criteria

- Phases 03–05 pass; transition outputs and clock ticks are deterministic and fixture-backed.
- Phase 00 resolved fork-choice generation, vote-promotion boundary, safe-target rule, tie-break order, pruning retention, and finality derivation from the selected snapshot.
- Every legacy consensus file in the fork-choice/finality set has a delete/replace mapping.

## Exact old and new paths

Replace and then delete baseline fork-choice/finality files:

- `src/consensus/fork_choice.rs`, `src/consensus/finality.rs`, `src/consensus/performance.rs`, `src/consensus/mod.rs`, `src/consensus/README.md` (and post–Phase 02 copies under `crates/ethean-node/src/consensus/`).

Create fork-choice crate per [TARGET_WORKSPACE](../03-architecture/TARGET_WORKSPACE.md):

- `crates/ethean-fork-choice/Cargo.toml`, `README.md`, `src/lib.rs`, `src/error.rs`.
- `src/store/{mod.rs,state.rs,checkpoint.rs}`, `src/update/{mod.rs,block.rs,vote.rs,tick.rs}`, `src/head/{mod.rs,select.rs,tie_break.rs}`, `src/prune.rs`.

Update node integration (temporary until Phase 09):

- `crates/ethean-node/src/client.rs`, `integration/coordinator.rs`, `integration/sync_coordinator.rs`, `storage/checkpoints.rs`.

New fixtures/tests:

- `spec/fixtures/phase-06/fork-choice/`, `spec/fixtures/phase-06/finality/`, `spec/fixtures/phase-06/manifest.toml`, `crates/ethean-fork-choice/tests/`, `tests/interop/fork_choice.rs`.

Every hand-written source file is at most **2000 lines**; split by responsibility before review.

## Ordered tasks

1. Model the pinned fork-choice store: latest messages, justified/finalized checkpoints, pending/known vote promotion, and safe-target state exactly as specified.
2. Implement block import hooks that consume Phase 05 validated blocks and update store metadata without re-running transition logic.
3. Implement vote ingestion, promotion across the pinned interval/tick boundary, conflict rejection, and future/stale vote handling.
4. Implement safe-target computation and head selection with the pinned tie-break rule; return decisions without persisting or networking.
5. Implement finality updates derived only from the selected snapshot (3SF-mini tick/adjacency rules or pinned Goldfish equivalent); no Beacon epoch finality.
6. Implement pruning that preserves recovery, proof verification, and configured history invariants defined in Phase 00.
7. Wire the node to call the pure crate and treat its output as the sole head/finality authority; remove direct legacy module use.
8. Run positive and negative fixture suites for ticks, reorgs, equivocation, tie-breaks, safe-target changes, and finality regression attempts.
9. Delete legacy fork-choice/finality files and prove no LMD-GHOST/GRANDPA/prevote-precommit symbols remain in production paths.
10. Record minimized divergence traces when peer differential tests disagree; resolve against leanSpec before exit.

## Deletion obligations

- Delete `src/consensus/fork_choice.rs`, `finality.rs`, `performance.rs`, and the old `consensus/mod.rs` exports; no wrappers.
- Delete LMD-GHOST weighting, GRANDPA round state, prevote/precommit collections, fixed 32-unit validator weights, and `epoch * 32` mappings.
- Delete `RealBLSAggregator` use in finality paths and any production mock vote acceptance.
- Delete Beacon Casper-FFG justification/finalization thresholds and epoch-scoped checkpoint logic unless the pinned Lean generation explicitly redefines equivalent semantics under Lean names.
- Remove empty `crates/ethean-node/src/consensus/` directory shells after deletion.

## Security/spec risks

- Wrong vote-promotion timing splits head and safe-target across clients operating on the same blocks.
- Safe-target or tie-break differences cause silent reorgs or stuck finality.
- Pruning too aggressively loses data needed for restart, proof verification, or slashing evidence in later phases.
- Retaining prevote/precommit or epoch-finality code paths can activate Beacon behavior behind Lean types.
- Implementing an unpinned Goldfish variant creates non-interoperable finality claims.

## Positive and negative fixtures

- Positive: tick progression, vote promotion at interval boundaries, safe-target updates, head tie-break cases, justified/finalized progression, side-branch reorg, and pruning retention cases from the pinned generation.
- Negative: unknown ancestry, conflicting votes, future slots, stale targets, finality regression attempts, duplicate promotion, equivocation where forbidden, and head selection under missing safe target.
- Include fixtures that prove pruning does not remove data required by Phase 11 recovery scenarios when those bounds are pinned.

## Interop and differential tests

- Run all pinned leanSpec fork-choice and finality vectors; compare head root, safe target, justified checkpoint, and finalized checkpoint exactly.
- Replay identical block/vote sequences in at least two same-generation peer implementations when available.
- Disagreement is evidence triaged against leanSpec; production code must not add peer-specific branches.

## Validation commands

```powershell
cargo test -p ethean-fork-choice --locked
cargo test --test fork_choice --locked
cargo test --workspace fork_choice_determinism --locked
cargo clippy -p ethean-fork-choice --all-targets --locked -- -D warnings
rg -n "LMD-GHOST|GRANDPA|prevote|precommit|epoch \* 32|Casper|RealBLSAggregator" crates/ethean-fork-choice crates/ethean-node/src
Test-Path crates/ethean-node/src/consensus/fork_choice.rs
git diff --check
```

`Test-Path` on deleted legacy files must return `False`.

## Exit criteria

- All pinned fork-choice and finality fixtures pass with exact head/safe-target/checkpoint outputs.
- Legacy fork-choice/finality/consensus modules are deleted; production has one pinned algorithm implementation.
- No LMD-GHOST, GRANDPA, prevote/precommit, or Beacon epoch-finality behavior remains reachable.
- Fork-choice crate is I/O-free; every touched source file is at most 2000 lines.
- Phase lock cites the implemented generation (3SF-mini or Goldfish) with fixture proof.

## Rollback/data policy

- Revert the complete phase; do not retain a fork-choice selector or dual head source.
- Fork-choice snapshots persisted by the node use a new schema/network identifier; prototype fork metadata is rejected.
- Rolling back code must not replay votes against an older store generation without explicit schema support.

## Artifacts/evidence

- Fork-choice/finality fixture manifest, conformance report, pruning proof, minimized divergence traces, forbidden-symbol audit, and `docs/lean-consensus-migration-phase-06-fork-choice-finality.md`.
- Local note under `bazalinacaklar/` records any peer deviation from the pinned generation.

## Dependencies

- Requires Phases 00–05.
- Phase 07 consumes stable signing roots and slot view; Phase 08 consumes validated participation surfaces; Phase 09 wires duties to head/safe target; Phases 10–11 consume head/checkpoint semantics; Phases 12–13 qualify fleet and release behavior against this finality model.
