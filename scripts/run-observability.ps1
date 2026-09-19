# Start Prometheus + Grafana for Ethean long-run metrics (Windows).
$ErrorActionPreference = "Stop"

function Assert-DockerReady {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw @"
Docker CLI not found in PATH.
Install Docker Desktop, then reopen this terminal.
"@
    }
    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $null = & docker info 2>&1
    $exit = $LASTEXITCODE
    $ErrorActionPreference = $prev
    if ($exit -ne 0) {
        throw @"
Docker daemon is not running (cannot reach npipe://./pipe/docker_engine).
Start Docker Desktop, wait until it says Engine running, then re-run:
  .\scripts\run-observability.ps1
"@
    }
}

Assert-DockerReady
Set-Location (Join-Path $PSScriptRoot "..\deploy\observability")
& docker compose up -d
if ($LASTEXITCODE -ne 0) {
    throw "docker compose up -d failed (exit $LASTEXITCODE)."
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
Write-Host "Scrape target expects ethean on host :9100 (ethean start --until-signal --metrics)"
