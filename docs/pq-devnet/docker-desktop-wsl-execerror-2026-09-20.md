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

If even `wsl --update` fails with **Sistem dosyaya erişemiyor**, the Store
package is corrupt (`NeedsRemediation`). Do **not** keep retrying
`wsl --update` — follow
[wsl-needsremediation-repair-2026-09-20.md](wsl-needsremediation-repair-2026-09-20.md).

Short path (elevated PowerShell, Docker Desktop quit first):

```powershell
Get-AppxPackage MicrosoftCorporationII.WindowsSubsystemForLinux | Remove-AppxPackage
winget install --id Microsoft.WSL -e --accept-package-agreements --accept-source-agreements
# reboot, then:
wsl --version
```

Or: **Settings → Apps → Windows Subsystem for Linux → Advanced → Repair/Reset**.

Then start Docker Desktop; wait until status is **Running**; confirm
`docker info` has a Server section; run `.\scripts\run-observability.ps1`.

If Docker Desktop is not installed at all, install it from
[Docker’s Windows install page](https://docs.docker.com/desktop/setup/install/windows-install/)
after WSL works.

## Code side

`bin/ethean/src/observability.rs` treats compose failure as a warning and leaves
the process running so `:9100` stays usable.
