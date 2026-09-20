# leanSpec FC safe-target suite + reorg_total (2026-09-20)

## Goal

Expand fixture coverage past finality/reorg/LMD into safe-target and store-prune
vectors, and count competing-tip head moves for `ethean_fc_reorg_total`.

## What landed

### Fixture locks (`fc_runner_safe_target_tests.rs`)

| Vector | Result |
| --- | --- |
| `safe_target_does_not_advance_below_supermajority` | green |
| `safe_target_advances_incrementally_along_the_chain` | green |
| `odd_five…four_votes_advance_safe_target` | green |
| `odd_five…three_votes_hold…genesis` | green |
| `head_retreats_onto_shorter_justified_fork` | green |
| `equal_slot_justified_candidate_keeps_original_root` | green |
| `finalization_prunes_stale_aggregated_payloads` | green |

`ethean-spec-fixtures` lib tests: **40** green (was 33).

### Fork-choice store

- `reorg_total` — incremented in `update_head` when the new head is **not** a
  descendant of the previous head (chain extension is not a reorg)
- Accessor `ForkChoiceStore::reorg_total()` for future metrics wiring

## Still open

| Gap | Notes |
| --- | --- |
| `test_prune_finalized_orphaned_branch/*` | `blockRoots` got 7 want 8 (over-prune) |
| `finalized_safety` | cached `at_9` empty body vs payload snapshot |
| Node scrape | publish store `reorg_total` into `ethean_fc_reorg_total` when FC is owned in-process |
