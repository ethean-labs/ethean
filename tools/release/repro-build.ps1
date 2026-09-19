# Reproducible release build (Windows PowerShell)
# Usage: pwsh tools/release/repro-build.ps1
$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root

$OutDir = Join-Path $Root "artifacts\phase-13\repro"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "cargo build --release --locked (workspace members that compile)"
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (-not (Test-Path $cargo)) { $cargo = "cargo" }

# Build Lean library crates; ethean-node may still carry pre-existing compile debt.
& $cargo build --release --locked `
  -p ethean-primitives -p ethean-profile -p ethean-ssz -p ethean-types `
  -p ethean-crypto -p ethean-genesis -p ethean-transition -p ethean-fork-choice `
  -p ethean-validator -p ethean-network-wire -p ethean-network `
  -p ethean-storage -p ethean-sync -p ethean-rpc -p ethean-metrics

$Meta = @{
  recorded_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  rustc = (& rustc --version)
  cargo = (& $cargo --version)
  git_head = (git rev-parse HEAD)
  lock_sha256 = (Get-FileHash -Algorithm SHA256 Cargo.lock).Hash.ToLowerInvariant()
}
$Meta | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $OutDir "build-meta.json")
Write-Host "Wrote $OutDir\build-meta.json"
