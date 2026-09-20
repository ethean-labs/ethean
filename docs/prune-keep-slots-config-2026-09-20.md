# Configurable durable prune keep window (2026-09-20)

## Goal

Let operators tune how many slots below finalized stay on disk under
`--data-dir`, without rebuilding the client.

## What landed

| Piece | Role |
| --- | --- |
| `--prune-keep-slots SLOTS` | CLI override |
| `ETHEAN_PRUNE_KEEP_SLOTS` | Env when the flag is omitted |
| Default `256` | `KEEP_BELOW_FINALIZED` |
| `StartConfig::prune_keep_slots` | Passed into flush / prune |
| `ethean_durable_blocks_prune_keep_slots` | Gauge on each flush sample |

Priority: CLI → env → default (same pattern as bootnodes / fork digest).

```bash
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data \
  --prune-keep-slots 512

# or
export ETHEAN_PRUNE_KEEP_SLOTS=128
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data
```

## Still open

| Gap | Notes |
| --- | --- |
| Alert on prune stall while finalized climbs | Done — [durable-prune-stall-alert-2026-09-20.md](./durable-prune-stall-alert-2026-09-20.md) |
