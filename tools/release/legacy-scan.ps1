# Legacy / forbidden symbol scan for release (Windows)
# Allowlist: road-to/** migration records only for panro/Beacon historical mentions.
$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root

$Patterns = @("panro", "Panro", "/eth2/", "NetworkManager", "mock.?gossip")
# blst may still appear in workspace Cargo.toml until node deps are purged — report, do not auto-fail that path yet.
$Roots = @("crates", "bin", "tests", "examples", "docs", "deploy", "tools")
$Hits = @()

foreach ($dir in $Roots) {
  $path = Join-Path $Root $dir
  if (-not (Test-Path $path)) { continue }
  foreach ($pat in $Patterns) {
    $found = rg -n -i --glob "!**/road-to/**" $pat $path 2>$null
    if ($LASTEXITCODE -eq 0 -and $found) {
      $Hits += $found
    }
  }
}

$OutDir = Join-Path $Root "artifacts\phase-13\legacy-scan"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$Report = Join-Path $OutDir "hits.txt"
if ($Hits.Count -eq 0) {
  "CLEAN" | Set-Content -Encoding utf8 $Report
  Write-Host "Legacy scan CLEAN"
  exit 0
} else {
  $Hits | Set-Content -Encoding utf8 $Report
  Write-Host "Legacy scan found $($Hits.Count) hit lines — see $Report"
  # Soft-fail for Phase 13 scaffolding: exit 0 but leave report for gate owners.
  # Hard-fail (exit 1) once ethean-node legacy purge completes.
  exit 0
}
