# Node safe_target / reorg_total wiring (2026-09-20)

## Goal

Stop stubbing `ethean_safe_target_slot` to head (or raw justified) without an
owner field, and publish `ethean_fc_reorg_total` from real head moves.

## What landed

- `chain_head.rs` — `advance_head(parent)` counts a reorg when the new block
  does not extend the prior head; `refresh_fc_view` sets `safe_target` from
  `latest_justified` until a live `ForkChoiceStore` owns the walk
- `ChainOwner` carries `safe_target` + `reorg_total`; snapshots expose them
- Gossip STF, local self-apply, dispatch import, persist restore, and local
  finality promotion call `advance_head` / `refresh_fc_view`
- Metrics: `safe_target_slot()` from the owner; `record_fc_reorg_total` sets
  the absolute counter each scrape
- Split durable/range helpers into `metrics/record_persist.rs` (≤2000 lines)

## Still open

- Embed `ethean_fork_choice::ForkChoiceStore` for true interval-3 safe-target
  and LMD reorg semantics (parent-only heuristic is interim)
- Re-fill leanSpec `at_9` / `dead_9` bodies for weight dump asserts
