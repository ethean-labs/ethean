# Monitoring (Lean)

Metrics prefix: `ethean_` (`ethean-metrics`).

## Live health APIs (bound with `ethean start`)

| URL | Expect |
| --- | --- |
| http://127.0.0.1:9100/healthz | `200` + `ok` |
| http://127.0.0.1:9100/readyz | `200` + `ready` (or `503` while booting) |
| http://127.0.0.1:9100/metrics | Prometheus text |

Disable with `--no-metrics`. Bind: `--metrics-address` / `--metrics-port`.

## Long-run

```text
ethean start --network pq-devnet-4 --until-signal
.\scripts\run-observability.ps1
```

- Grafana: http://localhost:3000 → **Ethean Lean Clients Dashboard**
- Prometheus: http://localhost:9090
- Targets: http://localhost:9090/targets

Lean JSON-RPC HTTP on `:5052` is not bound yet; operator health checks use the
metrics URLs above. In-process smoke still validates `GET /lean/v1/health`.

See root [README.md](../README.md#monitoring--metrics) and
[long-run-metrics-grafana-2026-09-20.md](./long-run-metrics-grafana-2026-09-20.md).
