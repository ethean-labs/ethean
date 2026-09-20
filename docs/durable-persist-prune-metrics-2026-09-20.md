# Durable persist and prune metrics (2026-09-20)

## Goal

Make `--data-dir` flush and finalized keep-window prune visible on `/metrics`
for soak runs and Grafana, without high-cardinality labels.

## What landed

| Metric | Kind | Meaning |
| --- | --- | --- |
| `ethean_durable_blocks_flushed_total` | counter | Applied blocks written on a successful flush |
| `ethean_durable_blocks_pruned_files_total` | counter | `blocks/*.ssz` files removed |
| `ethean_durable_blocks_pruned_redb_total` | counter | `ethean.redb` block rows removed |
| `ethean_durable_blocks_prune_floor_slot` | gauge | Floor after the last prune pass |

Wiring: `flush_chain_persist` records after `save_head` + `prune_below_floor`.
Families live in `ethean-metrics` (`record_durable_persist`).

Related: [durable-block-prune-finalized-keep-2026-09-20.md](./durable-block-prune-finalized-keep-2026-09-20.md),
[persist-applied-blocks-durable-2026-09-20.md](./persist-applied-blocks-durable-2026-09-20.md).

## Recipe

```powershell
cargo test -p ethean-metrics --lib
# with a long-run node: curl http://127.0.0.1:9100/metrics | findstr durable
```

## Still open

| Gap | Notes |
| --- | --- |
| Grafana panel for durable row | Done — [grafana-durable-persist-prune-panels-2026-09-20.md](./grafana-durable-persist-prune-panels-2026-09-20.md) |
| Configurable keep window | Still fixed at 256 |
