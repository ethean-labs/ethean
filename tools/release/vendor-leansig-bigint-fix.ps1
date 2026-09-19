# Vendor leanSig with num-bigint 0.5 to unblock Plonky3 (local only)

# leanSig pins num-bigint 0.4 while unpinned Plonky3 pulls 0.5 — BigUint types clash.
# This script clones into bazalinacaklar/ (gitignored) and patches Cargo.toml.

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Dest = Join-Path $Root "bazalinacaklar\leanSig-patched"
$Rev = "c08a3bae74b0d85379cab72dcbefa4091546ecbb"

if (Test-Path $Dest) {
  Write-Host "Removing existing $Dest"
  Remove-Item -Recurse -Force $Dest
}

git clone --depth 1 https://github.com/leanEthereum/leanSig.git $Dest
Push-Location $Dest
git fetch --depth 1 origin $Rev
git checkout $Rev
Pop-Location

$toml = Join-Path $Dest "Cargo.toml"
$text = Get-Content -Raw $toml
$patched = $text -replace 'num-bigint\s*=\s*"0\.4[^"]*"', 'num-bigint = "0.5"'
if ($patched -eq $text) {
  Write-Host "WARNING: num-bigint line not patched; check Cargo.toml manually"
} else {
  Set-Content -Path $toml -Value $patched -NoNewline
  Write-Host "Patched num-bigint -> 0.5 in $toml"
}

Write-Host ""
Write-Host "Next: point ethean-crypto at the path (local only, do not commit):"
Write-Host '  leansig = { path = "bazalinacaklar/leanSig-patched", package = "leansig", optional = true }'
Write-Host "Then: cargo check -p ethean-crypto --features leansig-backend"
