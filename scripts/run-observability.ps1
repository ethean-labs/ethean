# Start Prometheus + Grafana for Ethean long-run metrics (Windows).
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..\deploy\observability")
docker compose up -d
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
