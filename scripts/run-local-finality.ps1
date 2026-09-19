# Local finality long-run (Windows)

# Advances head/justified/finalized without public bootnodes.
# Metrics on :9100; optional Grafana via -MetricsStack.

param(
    [string]$Network = "pq-devnet-4",
    [int]$Validators = 4,
    [switch]$MetricsStack
)

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo build -p ethean --release

$argsList = @(
    "start",
    "--until-signal",
    "--network", $Network,
    "--validators", "$Validators"
)
if ($MetricsStack) { $argsList += "--metrics" }

Write-Host "ethean $($argsList -join ' ')"
Write-Host "Health: http://127.0.0.1:9100/healthz  Metrics: http://127.0.0.1:9100/metrics"
if ($MetricsStack) {
    Write-Host "Grafana: http://localhost:3000  Prometheus: http://localhost:9090"
}
ethean @argsList
