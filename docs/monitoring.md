# Monitoring (Lean)

Metrics prefix: `ethean_` (`ethean-metrics`). Lean HTTP API also serves
`GET /metrics` on the same listener as `/lean/v*` (default `:5052`).

## Live health APIs (bound with `ethean start`)

| URL | Expect |
| --- | --- |
| http://127.0.0.1:9100/healthz | `200` + `ok` |
| http://127.0.0.1:9100/readyz | `200` + `ready` (or `503` while booting) |
| http://127.0.0.1:9100/metrics | Prometheus text |
| http://127.0.0.1:5052/lean/v1/health | Hive health JSON |
| http://127.0.0.1:5052/lean/v1/validator/duties | Owned duty visibility |
| http://127.0.0.1:5052/lean/v0/events | Admin event poll (JSON) |

Scrape HTTP on `:9100` is on by default. Disable with `--no-metrics`.
Bind: `--metrics-address` / `--metrics-port`.

### Admin events SSE

Long-lived stream when the client asks for it (poll remains the default):

```text
curl -N -H "Accept: text/event-stream" http://127.0.0.1:5052/lean/v0/events
```

Pings every ~15s; idle close after ~60s without data; ends on shutdown.
Admin auth applies the same as the JSON poll route.

## Grafana + Prometheus UI

Needs **Docker Engine** + Compose. Start with:

```text
ethean start --network pq-devnet-4 --until-signal --metrics
```

Or separately: `./scripts/run-observability.sh`

| Service | URL |
| --- | --- |
| Grafana | http://localhost:3000 — **Ethean Lean Clients** + **Ethean Node Health** |
| Prometheus | http://localhost:9090 |
| Targets | http://localhost:9090/targets |

Without Docker, `:3000` / `:9090` stay down; `:9100/metrics` from the binary still works.

See root [README.md](../README.md#monitoring--metrics),
[metrics-flag-starts-grafana-prometheus-2026-09-20.md](observability/metrics-flag-starts-grafana-prometheus-2026-09-20.md),
and [long-run-metrics-grafana-2026-09-20.md](observability/long-run-metrics-grafana-2026-09-20.md),
[ethean-grafana-richer-monitors-2026-09-20.md](observability/ethean-grafana-richer-monitors-2026-09-20.md),
[proposal-duty-sse-events-2026-09-25.md](networking/proposal-duty-sse-events-2026-09-25.md).
