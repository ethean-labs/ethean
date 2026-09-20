# Blocks-by-range serve gap visibility (2026-09-20)

## What landed

Inbound `blocks_by_range` replies still skip missing slots (protocol allows a
partial list), but the serve path now **counts** gaps and logs them:

| Field | Meaning |
| --- | --- |
| `found` / `missing` | Hits vs holes in the requested slot walk |
| `cache_slots` | In-memory serve-cache size |
| Decode errors | `warn` instead of silent empty reply |

Cold peers that answer with zero bodies for `count > 0` emit
`blocks-by-range serve incomplete` so deep-lag catch-up debugging is visible
without inventing durable store seeding yet.

## Tests

```bash
cargo test -p ethean-network --lib collect_ --features libp2p-quic
```

## Follow-ups

- Seed serve cache from durable `--data-dir` when present — **landed**
  ([serve-cache-seed-from-data-dir-2026-09-20.md](serve-cache-seed-from-data-dir-2026-09-20.md))
- Metric counters for found/missing (observability scrape) — **landed**
  ([range-serve-seed-metrics-2026-09-20.md](../observability/range-serve-seed-metrics-2026-09-20.md))
- Operator A2/A3 still required for a live mesh
