# Grafana No data / 1970 date fixes (2026-09-20)

## What was wrong

1. **1970 start/genesis dates** — Grafana `dateTimeAsIso` expects **milliseconds**.
   Queries now use `ethean_start_time_seconds * 1000` (and genesis likewise).
2. **No data on validators / aggregator / lags / …** — those gauges exist in
   `ethean-metrics-v2`, but Grafana was scrapeing an older binary that only
   exported the original slot gauges. Rebuild + restart the node.
3. **Peers = 0** — solo runs with empty bootnodes are expected. Peer count now
   reads the **QuicSwarm** connection map (not the empty score table). New
   **Bootnodes** panel shows configured dial targets (`ethean_bootnode_count`).

## Operator reload

```text
cargo build -p ethean --release
ethean start --until-signal --network pq-devnet-4 --data-dir test-devnet --metrics
```

Confirm scrape:

```text
curl -s http://127.0.0.1:9100/metrics | findstr ethean_validator_count
curl -s http://127.0.0.1:9100/metrics | findstr ethean_bootnode_count
```

Restart Grafana compose if the dashboard JSON was already mounted, or wait for
provisioning refresh (folder **Ethean** → Lean Clients + Node Health).
