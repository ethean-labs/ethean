# Start Prometheus + Grafana for Ethean long-run metrics (Windows).
$ErrorActionPreference = "Stop"

$docker = Get-Command docker -ErrorAction SilentlyContinue
if (-not $docker) {
    Write-Host @"
ERROR: 'docker' is not on PATH.

Grafana (:3000) and Prometheus (:9090) are NOT started by:
  ethean start --until-signal --network …

Install/start Docker Desktop, open a new shell, then re-run this script.
Meanwhile the node scrape endpoint is only:
  http://127.0.0.1:9100/metrics
"@
    exit 1
}

Set-Location (Join-Path $PSScriptRoot "..\deploy\observability")
docker compose up -d
if ($LASTEXITCODE -ne 0) {
    Write-Host "docker compose failed (is Docker Desktop running?)"
    exit $LASTEXITCODE
}
Write-Host "Grafana: http://localhost:3000  Prometheus: http://localhost:9090"
Write-Host "Scrape target expects ethean on host :9100 (ethean start --until-signal)"
