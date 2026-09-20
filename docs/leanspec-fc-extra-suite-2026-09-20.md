# leanSpec FC extra suite expansion (2026-09-20)

## Goal

Lock more green leanSpec fork-choice vectors beyond the safe-target / prune
batch without waiting on `finalized_safety` weight gaps.

## What landed

New module `fc_runner_extra_tests.rs` (cache-gated):

| Suite | Vector |
| --- | --- |
| safe_target | follows heavier fork; ignores known pool at interval 3 |
| safe_target_supermajority | odd-seven 5 advance / 4 hold |
| equivocation | same-slot attesters count once |
| store_pruning | stale attestation signatures pruned |
| attestation_source_divergence | honest head-chain source; justified self-heal |
| fork_choice_reorgs | newly justified reorg; deep chain-split depth |
| checkpoint_sync | non-genesis anchor consistent |
| finalized_safety | `losing_fork_higher_finalized_does_not_latch` |

`ethean-spec-fixtures` lib: **54** green (was 42).

## Still open

| Gap | Notes |
| --- | --- |
| `test_fork_above_finalized_wins…` / `heavier_fork_below…` | `blockWeights` want 0 got 6 (`at_9` empty body vs 7-vote snapshot) |
| `test_tick_interval_0_skips_acceptance…` | `safeTargetRoot` mismatch |
| Node metrics | publish store `reorg_total` / real safe-target |
