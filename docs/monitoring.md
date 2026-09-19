# Monitoring (Lean)

Metrics prefix: `ethean_` (`ethean-metrics`).

HTTP scrape (default on): `http://127.0.0.1:9100/metrics` plus `/healthz` and `/readyz`.
Disable with `ethean start --no-metrics`.

Long-run:

```text
ethean start --network pq-devnet-4 --until-signal
.\scripts\run-observability.ps1
```

Grafana dashboard: `deploy/observability/grafana/dashboards/ethean-lean-clients.json`
(head / justified / finalized / current slot — Shariq-style panels).

Lean RPC health: `ethean-rpc` under `/lean/v1/…` only. Do not revive Beacon metric names.
