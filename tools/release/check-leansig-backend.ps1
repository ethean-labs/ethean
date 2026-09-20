# Local-only: vendor leanSig (num-bigint 0.5) + cargo check/test with leansig-backend.
# Uses a gitignored .cargo/config.toml [patch]; restores Cargo.lock afterward.
# Do not commit path overrides or the patched lockfile.

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root

$Dest = Join-Path $Root "bazalinacaklar\leanSig-patched"
$CargoDir = Join-Path $Root ".cargo"
$Config = Join-Path $CargoDir "config.toml"
$Lock = Join-Path $Root "Cargo.lock"
$LockBackup = Join-Path $Root "Cargo.lock.leansig-check.bak"
$RunTests = $args -contains "-Test"

if (-not (Test-Path (Join-Path $Dest "Cargo.toml"))) {
  Write-Host "Vendor tree missing; running vendor-leansig-bigint-fix.ps1"
  & (Join-Path $PSScriptRoot "vendor-leansig-bigint-fix.ps1")
}

New-Item -ItemType Directory -Force -Path $CargoDir | Out-Null
if (Test-Path $Config) {
  Write-Host "Replacing existing $Config"
}
@'
# LOCAL ONLY — written by tools/release/check-leansig-backend.ps1; do not commit.
[patch."https://github.com/leanEthereum/leanSig"]
leansig = { path = "bazalinacaklar/leanSig-patched" }
'@ | Set-Content -Path $Config -Encoding utf8

Copy-Item -Force $Lock $LockBackup

$exit = 0
try {
  Write-Host "cargo check -p ethean-crypto --features leansig-backend"
  cargo check -p ethean-crypto --features leansig-backend
  if ($LASTEXITCODE -ne 0) { throw "cargo check failed ($LASTEXITCODE)" }

  if ($RunTests) {
    Write-Host "cargo test -p ethean-crypto --features leansig-backend --lib"
    cargo test -p ethean-crypto --features leansig-backend --lib
    if ($LASTEXITCODE -ne 0) { throw "cargo test failed ($LASTEXITCODE)" }
  }
  Write-Host "leansig-backend OK against local vendor patch"
}
catch {
  Write-Error $_
  $exit = 1
}
finally {
  Remove-Item -Force $Config -ErrorAction SilentlyContinue
  if (Test-Path $LockBackup) {
    Move-Item -Force $LockBackup $Lock
  }
  Write-Host "Restored Cargo.lock; removed .cargo/config.toml"
}

exit $exit
