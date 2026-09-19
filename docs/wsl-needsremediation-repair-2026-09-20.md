# WSL NeedsRemediation — wsl.exe cannot start (2026-09-20)

## Symptom (admin PowerShell)

```text
PS C:\WINDOWS\system32> wsl --update
Program 'wsl.exe' failed to run: Sistem dosyaya erişemiyor
CategoryInfo: ResourceUnavailable: ApplicationFailedException
```

Same for `wsl --version`, `wsl -l -v`, and Docker Desktop
(`DockerDesktop/Wsl/ExecError`).

## Root cause on this host

| Check | Result |
| --- | --- |
| `C:\Windows\System32\wsl.exe` | Present, signed, WinBuild 10.0.22000.1281 |
| AppX `MicrosoftCorporationII.WindowsSubsystemForLinux` | **1.3.15.0 — Status: Modified, NeedsRemediation** |
| `LxssManager` service | Stopped; start fails |
| Current winget package | `Microsoft.WSL` **2.7.13** (inbox package is stale/corrupt) |
| Agent shell elevation | Not admin (`dism` → 740) |

`wsl --update` never runs: the launcher cannot load the remediating Store
package. This is a **Windows/WSL install corruption**, not an Ethean bug.

## Repair (run in elevated PowerShell)

Quit Docker Desktop first (tray → Quit Docker Desktop).

```powershell
# 1) Remove broken Store package (current user)
Get-AppxPackage MicrosoftCorporationII.WindowsSubsystemForLinux |
  Remove-AppxPackage

# 2) Install current WSL from winget
winget install --id Microsoft.WSL -e --accept-package-agreements --accept-source-agreements

# 3) Enable optional features if missing
dism.exe /Online /Enable-Feature /FeatureName:VirtualMachinePlatform /All /NoRestart
dism.exe /Online /Enable-Feature /FeatureName:Microsoft-Windows-Subsystem-Linux /All /NoRestart

# 4) Reboot, then verify
wsl --status
wsl --version
```

If `Remove-AppxPackage` fails, use:

**Settings → Apps → Installed apps → Windows Subsystem for Linux →
Advanced options → Repair**, then **Reset**, then reboot and `winget install
Microsoft.WSL`.

Fallback MSI (if winget blocked):
https://github.com/microsoft/WSL/releases (latest `wsl.x64.msi` or installer).

## After WSL works

1. Start Docker Desktop → wait for Engine running.
2. `docker info` shows a Server section.
3. `.\scripts\run-observability.ps1`

## Without WSL (Ethean still usable)

```powershell
ethean start --until-signal --network pq-devnet-4 --local-finality
curl http://127.0.0.1:9100/metrics
```

Grafana/Prometheus stay unavailable until WSL + Docker Engine are healthy.
