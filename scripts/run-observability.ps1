# Start Prometheus + Grafana for Ethean long-run metrics (Windows).
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..\deploy\observability")
docker compose up -d
Write-Host "Grafana: http://localhost:3000  Prometheus: http://localhost:9090"
Write-Host "Scrape target expects ethean on host :9100 (ethean start --until-signal)"
