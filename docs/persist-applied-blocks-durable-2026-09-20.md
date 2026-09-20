# Persist applied blocks for data-dir range serve (2026-09-20)

## Problem

`--data-dir` flush only wrote the optional `pending_block_gossip` blob. Synced /
gossip-imported heads advanced state but left `blocks/*.ssz` and `ethean.redb`
block rows empty, so restart serve-cache seeding stayed cold.

## What landed

| Piece | Change |
| --- | --- |
| `ChainOwner::durable_blocks` | Cap-64 queue of `(root, SSZ)` |
| `remember_durable_block` | Dedup by root; used on apply paths |
| `save_head` | Writes all queued (+ pending) blobs to SSZ + redb |
| `flush_chain_persist` | Clears queue after successful flush |
| Call sites | Gossip ingest, blocks-by-root sync, orphan drain, local propose |

Together with serve-cache seed (0.1.29), a restarted node can answer
blocks-by-range for history it previously imported or proposed.

## Tests

```bash
cargo test -p ethean-node --lib chain_persist
cargo test -p ethean-node --lib blocks_sync
```

## Follow-ups

- Cap / prune on-disk `blocks/` by finalized slot
- Observability counter for durable_blocks flushed
