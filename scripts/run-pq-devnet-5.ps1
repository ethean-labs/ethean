# Ready-path runner for pq-devnet-5 (needs operator bootnodes / fork digest).
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo build -p ethean --release
ethean start --until-signal --network pq-devnet-5 --metrics @args
