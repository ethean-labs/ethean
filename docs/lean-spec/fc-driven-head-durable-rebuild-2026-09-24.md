# FC-driven head + durable store rebuild (2026-09-24)

## Why

After wiring a live structural `ForkChoiceStore`, the node still imported only
linear extensions (`parent == head`) and could not rebuild the store after a
non-genesis `--data-dir` resume. Competing tips never entered the tree, and
resume dropped FC entirely.

## What landed

- `sync_from_fork_choice` copies **store head** (LMD tip) onto `ChainOwner`.
- `can_import_parent` / `pre_state_for_parent` admit any parent known to the
  store (not only the linear tip).
- Gossip / sync STF import uses those helpers; after `fc_on_block` the owner
  tip follows the store (imported block may lose the LMD race).
- `blocks_sync` orphans when the parent is unknown to FC; drain walks all
  known store roots.
- `chain_fc_rebuild.rs` replays durable blobs from genesis into a new store;
  falls back to tip-anchor when ancestors were pruned.
- Data-dir open clears the genesis-only store before restore/rebuild so the
  tip is not synced back to genesis.
- Local-finality checkpoint promotion is skipped when FC is live (store owns
  justified/finalized from post-states).

## Deferred

- Upstream fixture re-fill for empty-body dumps.
- Hive / operator A2–A3 pins only.

## Verify

```text
cargo test -p ethean-fork-choice --lib
cargo test -p ethean-node --lib -- can_import_parent rebuild_replays
```
