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
| finalized_safety | losing latch; fork-above wins; heavier-below never wins |

`ethean-spec-fixtures` lib: **58** green (was 54).

## Still open

| Gap | Notes |
| --- | --- |
| Filled `at_9` / `dead_9` empty bodies | Python BlockSpec has 7 votes; JSON body empty — weight dumps gated; see [empty-body gate](leanspec-fc-finalized-safety-empty-body-gate-2026-09-20.md) |
| Node metrics | live FC `safe_target` / `reorg_total` (justified used as safe stand-in) |

Tick interval-0 acceptance is green — see
[leanspec-fc-tick-safe-snapshot-gate-2026-09-20.md](leanspec-fc-tick-safe-snapshot-gate-2026-09-20.md).
