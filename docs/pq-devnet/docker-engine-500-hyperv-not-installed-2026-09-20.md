# Docker engine HTTP 500 — Hyper-V not installed (2026-09-20)

## Symptom

```text
ethean start --until-signal --network pq-devnet-4 --metrics
INFO ethean::observability: starting Prometheus + Grafana via docker compose (--metrics)
request returned 500 Internal Server Error for API route and version
http://%2F%2F.%2Fpipe%2FdockerDesktopLinuxEngine/_ping, check if the server
supports the requested API version
```

This is **not** an API-version mismatch in the Docker CLI. The named pipe
`\\.\pipe\dockerDesktopLinuxEngine` exists, but the Linux engine behind it is
dead, so `_ping` is HTTP 500.

## Diagnosis on this host (2026-09-20)

| Check | Result |
| --- | --- |
| Docker Desktop 4.91.0 UI | Running |
| `docker` CLI | 29.8.0, context `desktop-linux` |
| `wsl --version` | 2.9.12.0 (CLI works) |
| `wsl -l -v` | Ubuntu distros listed, **Stopped** |
| `docker-desktop` WSL distro | Missing |
| `HypervisorPresent` | **False** |
| Backend log | `wsl --import-in-place docker-desktop … ext4.vhdx` → `HCS_E_HYPERV_NOT_INSTALLED` |
| Ethean `:9100/metrics` | Independent of Docker |

`docker info` can hang after printing the 500. Desktop recovers with
“no virtualization available” and keeps the API proxy up anyway.

## What it is not

- Not a Grafana/Prometheus port conflict
- Not an Ethean compose file bug
- Not “install Docker” — CLI and Desktop are already there
- Not the older `wsl.exe` “Sistem dosyaya erişemiyor” launcher corruption
  (that was [wsl-needsremediation-repair-2026-09-20.md](wsl-needsremediation-repair-2026-09-20.md))

## Operator fix (elevated PowerShell)

Quit Docker Desktop first.

```powershell
wsl.exe --install --no-distribution
dism.exe /Online /Enable-Feature /FeatureName:VirtualMachinePlatform /All /NoRestart
dism.exe /Online /Enable-Feature /FeatureName:Microsoft-Windows-Subsystem-Linux /All /NoRestart
```

Enable CPU virtualization in firmware if `HypervisorPresent` stays False.
**Reboot**, then:

```powershell
wsl --status
# Start Docker Desktop; wait until Engine is Running
docker info   # must show a Server section
.\scripts\run-observability.ps1
```

## Without Grafana (node still usable)

```powershell
ethean start --until-signal --network pq-devnet-4
curl http://127.0.0.1:9100/healthz
curl http://127.0.0.1:9100/metrics
```

`--metrics` is best-effort: compose failure is a warning and the process
continues. `scripts/run-observability.ps1` now times out `docker info` at 15s
and prints this Hyper-V hint instead of hanging.
