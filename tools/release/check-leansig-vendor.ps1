# Check leanSig vendor overlay is present and applyable (Windows)

# Does not mutate crates/crypto/Cargo.toml. Verifies the committed patch and,
# when bazalinacaklar/leanSig-patched exists, that num-bigint is 0.5.

$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Patch = Join-Path $Root "vendor\leansig\num-bigint-0.5.patch"
$Dest = Join-Path $Root "bazalinacaklar\leanSig-patched\Cargo.toml"

if (-not (Test-Path $Patch)) {
  Write-Host "MISSING patch: $Patch"
  exit 1
}
Write-Host "FOUND patch $Patch"

if (Test-Path $Dest) {
  $line = Select-String -Path $Dest -Pattern 'num-bigint' | Select-Object -First 1
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
