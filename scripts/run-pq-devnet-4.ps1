# Run Ethean against the operational pq-devnet-4 label (Windows).
# Long-run path: wall clock until Ctrl-C, Prometheus /metrics on :9100.
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo build -p ethean --release
ethean start --until-signal --network pq-devnet-4 @args
