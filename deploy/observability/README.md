# Observability (Prometheus + Grafana)

Local stack for Shariq-style long-run monitoring of Ethean.

## Prerequisites

1. Docker Engine + Docker Compose plugin
2. Ethean built: `cargo build -p ethean --release`

## Long-run node (pq-devnet-4)

```bash
ethean start --network pq-devnet-4 --until-signal
# or
./scripts/run-pq-devnet-4.sh
```

Default scrape: `http://127.0.0.1:9100/metrics`  
Disable with `--no-metrics`. Bind: `--metrics-address` / `--metrics-port`.

## Start Prometheus + Grafana

**Preferred:** pass `--metrics` on the node (starts Docker Compose automatically):

```bash
ethean start --until-signal --network pq-devnet-4 --metrics
```

**Or** start the stack alone:

```bash
./scripts/run-observability.sh
# or
cd deploy/observability
docker compose up -d
```

Requires Docker Engine + Compose. Without it, `:3000` / `:9090` stay down
even though `:9100/metrics` from the binary works.

`--metrics` is best-effort: the node still serves `:9100/metrics` while Grafana
stays down.

- Grafana: http://localhost:3000 (anonymous viewer)
- Dashboards: **Ethean Lean Clients Dashboard**, **Ethean Node Health**
  (Node Health includes durable flush / prune and blocks-by-range serve rows)
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
- With `--data-dir`: `ethean_durable_blocks_*` counters move on flush; prune floor
  climbs after finality (see Node Health durable row)
- `EtheanDurablePruneStall` fires if finalized climbs while the prune floor is
  stuck behind `finalized − keep` for 15m (durable mode only)

Solo Ethean without a peer mesh will still advance **current** wall slot; head/finality need gossip + aggregator peers for a full Shariq-style curve.
