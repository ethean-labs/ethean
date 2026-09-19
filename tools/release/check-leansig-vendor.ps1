# Check leanSig vendor overlay is present and pin-aligned (Windows)
#
# Does not mutate crates/crypto/Cargo.toml. Verifies:
# - committed num-bigint 0.5 patch exists
# - LEANSIG_REV in xmss/config.rs matches Cargo.toml git rev
# - when bazalinacaklar/leanSig-patched exists, num-bigint is 0.5

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Patch = Join-Path $Root "vendor\leansig\num-bigint-0.5.patch"
$DestToml = Join-Path $Root "bazalinacaklar\leanSig-patched\Cargo.toml"
$ConfigRs = Join-Path $Root "crates\crypto\src\xmss\config.rs"
$CryptoToml = Join-Path $Root "crates\crypto\Cargo.toml"
$ExpectedRev = "c08a3bae74b0d85379cab72dcbefa4091546ecbb"

if (-not (Test-Path $Patch)) {
  Write-Host "MISSING patch: $Patch"
  exit 1
}
Write-Host "FOUND patch $Patch"

if (-not (Test-Path $ConfigRs)) {
  Write-Host "MISSING $ConfigRs"
  exit 1
}
$config = Get-Content -Raw $ConfigRs
if ($config -notmatch [regex]::Escape($ExpectedRev)) {
  Write-Host "LEANSIG_REV mismatch in config.rs (expected $ExpectedRev)"
  exit 1
}
Write-Host "FOUND LEANSIG_REV $ExpectedRev in xmss/config.rs"

if (-not (Test-Path $CryptoToml)) {
  Write-Host "MISSING $CryptoToml"
  exit 1
}
$cargo = Get-Content -Raw $CryptoToml
if ($cargo -notmatch [regex]::Escape($ExpectedRev)) {
  Write-Host "leansig git rev mismatch in crates/crypto/Cargo.toml (expected $ExpectedRev)"
  exit 1
}
Write-Host "FOUND matching leansig git rev in Cargo.toml"

if (Test-Path $DestToml) {
  $line = Select-String -Path $DestToml -Pattern 'num-bigint' | Select-Object -First 1
  if ($line -and $line.Line -match '0\.5') {
    Write-Host "FOUND vendor tree with num-bigint 0.5"
    exit 0
  }
  Write-Host "Vendor tree present but num-bigint is not 0.5: $($line.Line)"
  Write-Host "Re-run: pwsh tools/release/vendor-leansig-bigint-fix.ps1"
  exit 2
}

Write-Host "No local vendor tree yet. Run: pwsh tools/release/vendor-leansig-bigint-fix.ps1"
Write-Host "Then point crates/crypto leansig dep at the path (local only)."
exit 0
