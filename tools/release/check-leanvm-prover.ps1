# Check leanVM prover IPC wiring (Windows)
#
# Does not spawn a prover. Verifies:
# - LEANVM_REV pin is recorded in aggregation bindings
# - statement wire module exists
# - optional ETHEAN_LEANVM_PROVER path existence (report only)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Bindings = Join-Path $Root "crates\crypto\src\aggregation\bindings.rs"
$Wire = Join-Path $Root "crates\crypto\src\aggregation\statement_wire.rs"
$Ipc = Join-Path $Root "crates\crypto\src\leanvm_ipc.rs"
$ExpectedPrefix = "e2592df4"

if (-not (Test-Path $Bindings)) {
  Write-Host "MISSING $Bindings"
  exit 1
}
$bindings = Get-Content -Raw $Bindings
if ($bindings -notmatch $ExpectedPrefix) {
  Write-Host "LEANVM_REV missing expected prefix $ExpectedPrefix"
  exit 1
}
Write-Host "FOUND LEANVM_REV prefix $ExpectedPrefix"

foreach ($p in @($Wire, $Ipc)) {
  if (-not (Test-Path $p)) {
    Write-Host "MISSING $p"
    exit 1
  }
  Write-Host "FOUND $p"
}

$envPath = $env:ETHEAN_LEANVM_PROVER
if ([string]::IsNullOrWhiteSpace($envPath)) {
  Write-Host "ETHEAN_LEANVM_PROVER unset (IPC binary not configured)"
  exit 0
}
if (Test-Path -LiteralPath $envPath -PathType Leaf) {
  Write-Host "FOUND prover binary at $envPath (protocol still not ready in-tree)"
  exit 0
}
Write-Host "ETHEAN_LEANVM_PROVER set but file missing: $envPath"
exit 2
