# Why localhost:3000 / :9090 stay blank (2026-09-20)

## Diagnosis on this machine

| Port | Listening? | Process |
| --- | --- | --- |
| `127.0.0.1:9100` | yes | `ethean` scrape HTTP (`/metrics`) |
| `localhost:3000` | **no** | Grafana container never started |
| `localhost:9090` | **no** | Prometheus container never started |

`docker` is **not on PATH** and Docker Desktop binaries were not found under the usual
install locations. So `--metrics` cannot run `docker compose up -d`.

## Not a port conflict

Changing Grafana/Prometheus to other ports would not help: nothing binds those ports
until Docker starts the containers. The compose file already maps `3000:3000` and
`9090:9090`.

## Fix

1. Install [Docker Desktop for Windows](https://docs.docker.com/desktop/setup/install/windows-install/).
2. Start Docker Desktop and wait until it says **Running**.
3. From the repo root:

```powershell
ethean start --until-signal --network pq-devnet-4 --metrics
# or only the UI stack:
.\scripts\run-observability.ps1
```

4. Open http://localhost:3000 and http://localhost:9090/targets  
   Keep using http://127.0.0.1:9100/metrics for the node scrape itself.
