# Monitoring (Lean)

Metrics prefix: `ethean_` (`ethean-metrics`).

## Live health APIs (bound with `ethean start`)

| URL | Expect |
| --- | --- |
| http://127.0.0.1:9100/healthz | `200` + `ok` |
| http://127.0.0.1:9100/readyz | `200` + `ready` (or `503` while booting) |
| http://127.0.0.1:9100/metrics | Prometheus text |

Scrape HTTP is on by default. Disable with `--no-metrics`.
Bind: `--metrics-address` / `--metrics-port`.

## Grafana + Prometheus UI

Needs **Docker Desktop** (or Engine + Compose). Start with:

```text
ethean start --network pq-devnet-4 --until-signal --metrics
```

Or separately: `.\scripts\run-observability.ps1`

| Service | URL |
| --- | --- |
| Grafana | http://localhost:3000 — **Ethean Lean Clients** + **Ethean Node Health** |
| Prometheus | http://localhost:9090 |
| Targets | http://localhost:9090/targets |

Without Docker, `:3000` / `:9090` stay down; `:9100/metrics` from the binary still works.

Lean JSON-RPC HTTP on `:5052` is not bound yet; operator health checks use the
metrics URLs above. In-process smoke still validates `GET /lean/v1/health`.

See root [README.md](../README.md#monitoring--metrics),
[metrics-flag-starts-grafana-prometheus-2026-09-20.md](./metrics-flag-starts-grafana-prometheus-2026-09-20.md),
and [long-run-metrics-grafana-2026-09-20.md](./long-run-metrics-grafana-2026-09-20.md),
[ethean-grafana-richer-monitors-2026-09-20.md](./ethean-grafana-richer-monitors-2026-09-20.md).
