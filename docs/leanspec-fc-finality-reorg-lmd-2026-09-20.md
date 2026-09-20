# leanSpec FC finality / reorg / LMD suites (2026-09-20)

## Goal

Expand the fork-choice fixture runner beyond rejection and gossip-aggregate
smoke into leanSpec finality, reorg, and LMD latest-message vectors.

## What landed

New module `fc_runner_finality_tests.rs` (cache-gated). Confirmed green:

| Suite | Fixture |
| --- | --- |
| LMD | `test_higher_slot_vote_replaces_lower_slot_vote` |
| LMD | `test_lexicographic_tiebreak_selects_larger_root_and_is_stable` |
| Reorg | `test_simple_one_block_reorg` |
| Reorg | `test_two_block_reorg_progressive_building` |
| Reorg | `test_three_block_deep_reorg` |
| Finality | `test_finalization_advances_mid_attestation_processing` |
| Head | `test_head_switches_to_heavier_fork` |
| Head | `test_head_selection_by_weight_not_depth` |
| Head | `test_block_that_justifies_reanchors_within_one_import` |
| Head | `test_duplicate_block_processed_idempotently` |

`cargo test -p ethean-spec-fixtures --lib fc_runner` → **22** tests (12 prior + 10).

## Still open

| Gap | Notes |
| --- | --- |
| `test_finalized_safety/*` | `blockWeights` expect `0` for some orphaned roots; store still reports weight |
| Deeper reorg / three-way / oscillation fixtures | Not yet pinned in CI |
| Exact payload-set prune vs leanSpec | Separate track |

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib fc_runner
```
