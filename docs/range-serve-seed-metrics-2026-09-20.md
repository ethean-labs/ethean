# Range-serve and serve-cache seed metrics (2026-09-20)

## Goal

Scrape blocks-by-range serve honesty and durable seed size without grepping
logs — closes follow-ups on serve-cache seed and gap-warn docs.

## What landed

| Metric | Kind | Source |
| --- | --- | --- |
| `ethean_blocks_by_range_serve_found_total` | counter (mirrored) | QuicSwarm atomics |
| `ethean_blocks_by_range_serve_missing_total` | counter (mirrored) | QuicSwarm atomics |
| `ethean_blocks_by_range_serve_cache_slots` | gauge | in-memory cache size |
| `ethean_serve_cache_seed_candidates` | gauge | boot seed from `--data-dir` |
| `ethean_serve_cache_seed_indexed` | gauge | boot seed indexed count |

Network stays free of `ethean-metrics`: atomics live on `QuicSwarm`; the node
copies them on each slot metrics refresh. Seed gauges are set once after
`seed_facade_from_data_dir`.

## Recipe

```powershell
cargo test -p ethean-metrics --lib
cargo test -p ethean-network --lib collect_ --features libp2p-quic
# curl http://127.0.0.1:9100/metrics | findstr range_serve
```

## Still open

| Gap | Notes |
| --- | --- |
| Slot index table for faster seed | Full blob decode at boot remains |
| Grafana panels for range-serve row | Optional next |
