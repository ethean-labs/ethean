# Grafana range-serve / seed panels (2026-09-20)

## Goal

Surface the 0.1.37 range-serve and serve-cache seed scrapes on Node Health so
deep-lag debugging does not require curling `/metrics`.

## What landed

| Panel | Metric / query |
| --- | --- |
| Range cache slots | `ethean_blocks_by_range_serve_cache_slots` |
| Seed candidates / indexed | `ethean_serve_cache_seed_*` |
| Range found / missing | `ethean_blocks_by_range_serve_*_total` |
| Serve rate | `rate(...found/missing[5m])` |

File: `deploy/observability/grafana/dashboards/ethean-node-health.json` (version 4).

## Still open

| Gap | Notes |
| --- | --- |
| Slot index table for faster seed | Full blob decode at boot remains |
