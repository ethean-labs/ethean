# Durable block prune (finalized keep window) (2026-09-20)

## Goal

Bound `--data-dir` growth after applied blocks are persisted, without wiping the
recent history peers may still request via blocks-by-range.

## What landed

| Piece | Role |
| --- | --- |
| `crates/node/src/block_prune.rs` | `KEEP_BELOW_FINALIZED=256`, `prune_floor`, `prune_below_floor` |
| `client_data_dir::flush_chain_persist` | After successful `save_head`, prune SSZ + `ethean.redb` |

Floor slot: `finalized.saturating_sub(256)`. Blobs with `slot < floor` are
removed from `blocks/*.ssz` and the redb `ethean_blocks` table. Floor `0`
skips pruning (early chain / no finality yet).

Prune failures are logged and do not fail the flush.

## Recipe

```powershell
cargo test -p ethean-node --lib block_prune
```

## Still open

| Gap | Notes |
| --- | --- |
| Configurable keep window | Done — [prune-keep-slots-config-2026-09-20.md](prune-keep-slots-config-2026-09-20.md) |
| Genesis catch-up honesty past prune | Peers below floor must sync elsewhere |
