# Durable prune stall Prometheus alert (2026-09-20)

## Goal

Warn when `--data-dir` finality advances but the prune floor stops moving, so
disk growth or stuck flush errors surface in the observability stack.

## What landed

| Piece | Role |
| --- | --- |
| `EtheanDurablePruneStall` | `deploy/observability/alerting/rules/ethean.yml` |

Fires when all hold for 15m:

1. `ethean_durable_blocks_prune_keep_slots > 0` (at least one durable flush)
2. `finalized - keep > floor` (floor behind the keep window)
3. Finalized increased in the last 15m
4. Floor did not increase in the last 15m

Ephemeral / no `--data-dir` stays quiet because the keep gauge remains 0 until
the first durable flush.

## Still open

| Gap | Notes |
| --- | --- |
| Alertmanager receiver routing | Compose stack evaluates rules; paging is optional |
