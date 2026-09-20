# Grafana durable flush / prune row (2026-09-20)

## Goal

Surface the 0.1.32 durable persist metrics on the provisioned Node Health
dashboard so soak operators do not need to curl `/metrics`.

## What landed

| Panel | Metric / query |
| --- | --- |
| Prune floor slot | `ethean_durable_blocks_prune_floor_slot` |
| Durable flushed | `ethean_durable_blocks_flushed_total` |
| Pruned SSZ files | `ethean_durable_blocks_pruned_files_total` |
| Pruned redb rows | `ethean_durable_blocks_pruned_redb_total` |
| Flush / prune rate | `rate(...[5m])` for flushed + both prune counters |

File: `deploy/observability/grafana/dashboards/ethean-node-health.json` (uid
`ethean-node-health`, version 2). Restart or remount Grafana compose to pick up
provisioned JSON.

## Still open

| Gap | Notes |
| --- | --- |
| Configurable keep window | Still fixed at 256 |
| Alert on prune stall while finalized climbs | Optional later |
