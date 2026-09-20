# Seed blocks-by-range serve cache from `--data-dir` (2026-09-20)

## What landed

After QuicSwarm binds, a node started with `--data-dir` indexes durable block
blobs into the in-memory range serve cache:

| Source | Loader |
| --- | --- |
| `ethean.redb` `ethean_blocks` | `chain_redb::load_all_blocks` |
| `blocks/<root>.ssz` | `persist_ssz::load_block_ssz_dir` |

`serve_cache_seed` dedupes by root (redb first), decodes slot from
`SignedBlock` / bare `Block`, then calls `put_block_at_slot`. Boot logs
`seeded blocks-by-range serve cache from data-dir` with candidate/indexed counts.

Cold peers that previously replied empty on deep-lag range requests can now
serve restarted history that was flushed via `save_head` / `save_block_ssz`.

## Tests

```bash
cargo test -p ethean-node --lib serve_cache_seed
```

## Follow-ups

- Slot index table for faster range walks (today: full blob decode at seed)
- Metric for indexed count on scrape
- Operator A2/A3 still required for a live mesh
