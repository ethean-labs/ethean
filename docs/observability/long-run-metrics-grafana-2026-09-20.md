# Long-run pq-devnet-4 + Prometheus/Grafana (2026-09-20)

## Goal

Match Ream/ethlambda long-run practice: keep a node up for hours/days and watch
head / justified / finalized / current slot on Grafana.

## What landed

1. `ethean-metrics` serves `/metrics`, `/healthz`, `/readyz`.
2. Slot gauges: `ethean_head_slot`, `ethean_justified_slot`, `ethean_finalized_slot`,
   `ethean_slot_current`, `ethean_peer_count`, `ethean_start_time_seconds`.
3. `ethean start` enables scrape on `127.0.0.1:9100` by default; refresh each duty tick.
4. `deploy/observability/docker-compose.yml` + **Ethean Lean Clients Dashboard**.
5. Helpers: `scripts/run-observability.{ps1,sh}`, `run-pq-devnet-4` already uses `--until-signal`.

## Operator commands

```powershell
# Terminal A — long-run node
cargo build -p ethean --release
ethean start --network pq-devnet-4 --until-signal

# Terminal B — scrape stack
.\scripts\run-observability.ps1
```

Open http://localhost:3000 → **Ethean Lean Clients Dashboard** (and
**Ethean Node Health** for readiness / lag / peers). See
[ethean-grafana-richer-monitors-2026-09-20.md](./ethean-grafana-richer-monitors-2026-09-20.md).

## Honest limits vs a multi-client mesh board

- Solo Ethean advances **wall current slot** immediately.
- Head / justified / finalized climb like a healthy multi-validator run only when
  peers + aggregator + STF/finality path are live (local mesh or operator paste).
- Finality stall (head climbing, finalized stuck) is exactly what the lag panels
  catch before scaling the mesh.
