# `--metrics` starts Prometheus + Grafana (2026-09-20)

## Problem

`ethean start --until-signal --network pq-devnet-5` already served scrape metrics at
`http://127.0.0.1:9100/metrics`, but README links to Grafana (`:3000`) and Prometheus
(`:9090`) stayed blank because those UIs live in **Docker Compose**, not inside the
binary. Users assumed a missing `--metrics` flag; scrape was already on by default.

## Fix

| Flag | Behaviour |
| --- | --- |
| *(default)* | Scrape HTTP on `:9100` |
| `--metrics` | Also runs `docker compose up -d` in `deploy/observability` |
| `--no-metrics` | Disables scrape HTTP |

Helper: `bin/ethean/src/observability.rs` (finds compose dir via cwd / exe / `ETHEAN_ROOT`).

## Run

```bash
ethean start --until-signal --network pq-devnet-4 --metrics
# Docker Desktop must be running
```

Then open http://localhost:3000 and http://localhost:9090/targets.
