# Grafana / Prometheus blank while ethean runs (2026-09-20)

## Symptom

```text
ethean start --until-signal --network pq-devnet-5
```

- http://127.0.0.1:9100/metrics → works
- http://localhost:3000 → blank / dead
- http://localhost:9090 → blank / dead

## Cause

`ethean` embeds only the **Prometheus scrape HTTP** on `:9100`. Grafana and
Prometheus UIs live in `deploy/observability/docker-compose.yml` and must be
started separately with Docker:

```powershell
.\scripts\run-observability.ps1
```

On this host, `docker` was not on `PATH` and nothing listened on ports 3000/9090
(only 9100). Blank browser tabs are “nothing bound”, not a Grafana theme bug.

## Fix

1. Install and start **Docker Desktop**.
2. Keep `ethean start --until-signal …` running (metrics on `:9100`).
3. New terminal: `.\scripts\run-observability.ps1`
4. Open http://localhost:3000 and http://localhost:9090/targets (`ethean` UP).

README Monitoring section and the observability scripts now state this explicitly.
