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

**Preferred:** pass `--metrics` on the node (starts Docker Compose automatically):

```powershell
ethean start --until-signal --network pq-devnet-4 --metrics
```

**Or** start the stack alone:

```powershell
.\scripts\run-observability.ps1
# or
cd deploy/observability
docker compose up -d
```

Requires Docker Desktop (or Engine + Compose). Without it, `:3000` / `:9090` stay down
even though `:9100/metrics` from the binary works.

### Windows: `npipe://./pipe/docker_engine` / file not found

That error means the **Docker CLI is installed but the daemon is stopped**
(Docker Desktop not running, still starting, or backend crash-looping). Fix:

1. Start **Docker Desktop** from the Start menu (or
   `%LOCALAPPDATA%\Programs\DockerDesktop\Docker Desktop.exe`).
2. Wait until the tray icon reports the engine is running.
3. Re-run `.\scripts\run-observability.ps1` (it fails fast if the daemon is down).

If Desktop opens but Engine never comes up, check WSL:

```powershell
wsl -l -v
# If that fails ("cannot access the file" / similar): Admin PowerShell
wsl --install
# or
wsl --update
```

Then reboot and start Docker Desktop again. Without a working WSL2 distro,
`com.docker.backend` exits and Grafana/Prometheus never bind `:3000` / `:9090`.

### Windows: HTTP 500 on `dockerDesktopLinuxEngine/_ping`

```text
request returned 500 Internal Server Error for API route and version
http://%2F%2F.%2Fpipe%2FdockerDesktopLinuxEngine/_ping
```

The CLI and Desktop UI are present; the **Linux engine is not**. Docker is
proxying `_ping` with HTTP 500 because WSL2 cannot create a VM. Typical host
log: `HCS_E_HYPERV_NOT_INSTALLED` / `HypervisorPresent = False`.

Fix (elevated PowerShell), then **reboot**:

```powershell
wsl.exe --install --no-distribution
dism.exe /Online /Enable-Feature /FeatureName:VirtualMachinePlatform /All /NoRestart
```

Enable CPU virtualization in firmware if it is off. After reboot, start Docker
Desktop, wait until Engine is **Running**, confirm `docker info` has a Server
section, then re-run `--metrics` or `.\scripts\run-observability.ps1`.

`--metrics` is best-effort: the node still serves `:9100/metrics` while Grafana
stays down.

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
