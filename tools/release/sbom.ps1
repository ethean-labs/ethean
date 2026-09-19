# SBOM generation stub (Phase 13)
# Prefer: cargo install cargo-cyclonedx ; cargo cyclonedx
# Also run: cargo deny check (when deny.toml is present)
$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root
$OutDir = Join-Path $Root "artifacts\phase-13\sbom"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$Note = @"
SBOM generation is gated on installing cargo-cyclonedx (or equivalent) in CI.
Until then this stub records the Cargo.lock SHA-256 as the dependency fingerprint.
"@
Set-Content -Encoding utf8 (Join-Path $OutDir "README.txt") $Note
$Hash = (Get-FileHash -Algorithm SHA256 Cargo.lock).Hash.ToLowerInvariant()
Set-Content -Encoding utf8 (Join-Path $OutDir "cargo.lock.sha256") $Hash
Write-Host "Wrote lock fingerprint $Hash"
