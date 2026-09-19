# Start Prometheus + Grafana for Ethean long-run metrics (Windows).
# Requires a healthy Docker Desktop + WSL2 backend.
$ErrorActionPreference = "Stop"

function Assert-DockerReady {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw @"
Docker CLI not found in PATH.
Install Docker Desktop, then reopen this terminal.

Until Docker works, run ethean alone and scrape:
  curl http://127.0.0.1:9100/metrics
  curl http://127.0.0.1:9100/healthz
"@
    }
    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $info = & docker info 2>&1 | Out-String
    $exit = $LASTEXITCODE
    $ErrorActionPreference = $prev
    if ($exit -ne 0) {
        $hint = if ($info -match 'wsl|WSL|Sistem dosyaya') {
            @"

This looks like a Docker Desktop / WSL failure (wsl.exe exit 1).
Fix on Windows:
  1. Quit Docker Desktop fully
  2. In admin PowerShell: wsl --shutdown
  3. Optional: wsl --update
  4. Start Docker Desktop; wait until Engine is running
  5. Re-run: .\scripts\run-observability.ps1

Ethean does NOT need Grafana to advance head — use:
  ethean start --until-signal --network pq-devnet-4
  curl http://127.0.0.1:9100/metrics
"@
        } else {
            @"

Start Docker Desktop, wait until it says Engine running, then re-run:
  .\scripts\run-observability.ps1
"@
        }
        throw "Docker daemon not ready.$hint"
    }
}

Assert-DockerReady
Set-Location (Join-Path $PSScriptRoot "..\deploy\observability")
$prev = $ErrorActionPreference
$ErrorActionPreference = "Continue"
& docker compose up -d 2>&1 | Write-Host
$composeExit = $LASTEXITCODE
$ErrorActionPreference = $prev
if ($composeExit -ne 0) {
    throw @"
docker compose up -d failed (exit $composeExit).

If the error mentions wsl.exe / 'Sistem dosyaya erişemiyor', repair WSL/Docker Desktop
(see Assert-DockerReady hints above). Ethean metrics still work at :9100 without Grafana.
"@
}

Write-Host "Waiting for Prometheus..."
$ok = $false
for ($i = 0; $i -lt 30; $i++) {
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:9090/-/ready" -UseBasicParsing -TimeoutSec 2
        if ($r.StatusCode -eq 200) { $ok = $true; break }
    } catch { Start-Sleep -Seconds 1 }
}
if (-not $ok) {
    Write-Warning "Prometheus not ready yet; open http://localhost:9090 after Docker finishes pulling."
}
Write-Host "Grafana: http://localhost:3000  Prometheus: http://localhost:9090"
Write-Host "Scrape target expects ethean on host :9100 (ethean start --until-signal)"
