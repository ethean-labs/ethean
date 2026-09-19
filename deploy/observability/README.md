# Observability (Prometheus + Grafana)

Local stack for Shariq-style long-run monitoring of Ethean.

## Prerequisites

1. Docker Desktop (or Docker Engine + Compose)
2. Ethean built: `cargo build -p ethean --release`

## Long-run node (pq-devnet-4)

```powershell
ethean start --network pq-devnet-4 --until-signal
# or
.\scripts\run-pq-devnet-4.ps1
```

Default scrape: `http://127.0.0.1:9100/metrics`  
Disable with `--no-metrics`. Bind: `--metrics-address` / `--metrics-port`.

## Start Prometheus + Grafana

```powershell
.\scripts\run-observability.ps1
# or
cd deploy/observability
docker compose up -d
```

- Grafana: http://localhost:3000 (anonymous viewer)
- Dashboard: **Ethean Lean Clients Dashboard**
- Prometheus: http://localhost:9090

## Health check URLs

| URL | Expect |
| --- | --- |
| http://127.0.0.1:9100/healthz | `200` + `ok` |
| http://127.0.0.1:9100/readyz | `200` / `503` |
| http://127.0.0.1:9100/metrics | Prometheus text |
| http://localhost:3000 | Grafana |
| http://localhost:9090 | Prometheus |
| http://localhost:9090/targets | scrape UP |

## What healthy long-run looks like

Like the Ream/ethlambda Grafana panels:

- `ethean_slot_current` and `ethean_head_slot` climb over hours/days
- `ethean_justified_slot` / `ethean_finalized_slot` advance when finality works
- Flat finalized while head climbs = finality stall (debug before multi-day runs)

Solo Ethean without a peer mesh will still advance **current** wall slot; head/finality need gossip + aggregator peers for a full Shariq-style curve.
