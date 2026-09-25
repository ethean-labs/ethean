# leanSpec FC tick safe-target snapshot gating (2026-09-20)

## Diagnosis

`test_tick_interval_0_skips_acceptance_when_not_proposer` failed on
`storeSnapshot.safeTargetRoot` (got head, want genesis) even though:

- leanSpec Python `StoreChecks` for that vector never assert safe-target
- sibling `test_tick_interval_progression_through_full_slot` asserts
  `safe_target_slot=2` after the same 3/4 pending votes and advances
- our `on_tick` correctly steps every interval (update safe at within=3,
  accept at within=4)

Filled dump also lists all-zero `blockWeights` and empty payload pools at
time 19, which matches a filler that jumped the clock without running
intermediate interval actions. Current leanSpec `on_tick` loops
interval-by-interval; Ethean matches that.

## Fix

In `fc_checks.rs`:

- Enforce `storeSnapshot.safeTargetRoot` only when `checks` pin
  `safeTargetSlot` / `safeTargetRoot` / `safeTargetRootLabel`
- Also assert `checks.safeTargetSlot` / `safeTargetRoot` directly
- Ignore all-zero `blockWeights` dumps (non-authoritative)

Locked green: both tick-system vectors in `fc_runner_extra_tests.rs`.
Split payload-pool snapshot helpers into `fc_snapshot_payloads.rs`
(≤2000-line rule).

## Still open

- Regenerate filled `at_9` / `dead_9` bodies (see finalized_safety empty-body gate)
- Live `ForkChoiceStore` / `safe_target` / `reorg_total` in the node — landed
  later (see FC store / safe-target attest notes under `docs/lean-spec/`)
