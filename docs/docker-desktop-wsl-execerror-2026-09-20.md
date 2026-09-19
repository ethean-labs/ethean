# Docker Desktop WSL ExecError (2026-09-20)

## Symptom

```text
running wslexec: An error occurred while running the command.
DockerDesktop/Wsl/ExecError: c:\windows\system32\wsl.exe --version: exit status 1
(stderr: Sistem dosyaya erişemiyor., stdout: , wslErrorCode: DockerDesktop/Wsl/ExecError)
```

Same failure from a plain shell: `wsl --version` exits 1 with
`Sistem dosyaya erişemiyor` (“The system cannot access the file”).

## What it means

Docker Desktop on Windows boots its Linux VM through WSL2. If `wsl.exe` cannot
run, compose cannot start Grafana/Prometheus. **This is a host WSL/Docker
problem, not an Ethean bug.**

On this machine at the time of writing:

| Check | Result |
| --- | --- |
| `wsl --version` | exit 1, file-access error |
| `docker` on PATH | missing |
| Ethean `:9100` scrape | independent of Docker/WSL |

## What still works

```powershell
ethean start --until-signal --network pq-devnet-4 --local-finality
# or with --metrics: node still runs; compose warn is non-fatal
curl -s http://127.0.0.1:9100/healthz
curl -s http://127.0.0.1:9100/metrics | findstr ethean_head_slot
```

Skip Grafana until WSL is healthy. See also
[grafana-prometheus-need-docker-2026-09-20.md](grafana-prometheus-need-docker-2026-09-20.md).

## Fix WSL / Docker Desktop (operator)

1. Quit Docker Desktop completely (tray icon → Quit).
2. Admin PowerShell:
   - `wsl --shutdown`
   - `wsl --update`
   - If still broken: **Settings → Apps → Windows Subsystem for Linux** repair,
     or reinstall WSL (`wsl --install` after optional feature enable).
3. Enable **Virtual Machine Platform** and **Windows Subsystem for Linux**
   (optional features), reboot if Windows asks.
4. Start Docker Desktop; wait until status is **Running**.
5. Confirm:
   - `wsl --version` prints a version
   - `docker info` succeeds
6. Then: `.\scripts\run-observability.ps1` or `ethean start ... --metrics`.

If Docker Desktop is not installed at all, install it from
[Docker’s Windows install page](https://docs.docker.com/desktop/setup/install/windows-install/)
after WSL works.

## Code side

`bin/ethean/src/observability.rs` treats compose failure as a warning and leaves
the process running so `:9100` stays usable.
