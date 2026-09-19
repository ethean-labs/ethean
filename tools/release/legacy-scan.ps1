# Legacy / forbidden symbol scan for release (Windows)
# Allowlist: road-to/** migration records only for panro/Beacon historical mentions.
$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root

function Collect-Hits([string[]]$Dirs, [string[]]$Patterns) {
  $acc = @()
  foreach ($dir in $Dirs) {
    $path = Join-Path $Root $dir
    if (-not (Test-Path $path)) { continue }
    foreach ($pat in $Patterns) {
      $found = rg -n --glob "!**/road-to/**" $pat $path 2>$null
      if ($LASTEXITCODE -eq 0 -and $found) {
        $acc += $found
      }
    }
  }
  return $acc
}

# Soft patterns (report; docs may still discuss retirement).
$SoftPatterns = @("panro", "Panro", "/eth2/", "NetworkManager", "mock.?gossip")
$SoftHits = Collect-Hits @("crates", "bin", "tests", "examples", "docs", "deploy", "tools") $SoftPatterns

# Hard patterns: BLS crates must not appear under active code manifests/sources.
$HardPatterns = @("\bblst\b", "\bblstrs\b", "\bbls12_381\b")
$HardHits = Collect-Hits @("crates", "bin") $HardPatterns

$OutDir = Join-Path $Root "artifacts\phase-13\legacy-scan"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$Report = Join-Path $OutDir "hits.txt"
$HardReport = Join-Path $OutDir "hard-hits.txt"

if ($HardHits.Count -eq 0) {
  "CLEAN" | Set-Content -Encoding utf8 $HardReport
} else {
  $HardHits | Set-Content -Encoding utf8 $HardReport
}

if ($SoftHits.Count -eq 0) {
  "CLEAN" | Set-Content -Encoding utf8 $Report
} else {
  $SoftHits | Set-Content -Encoding utf8 $Report
}

if ($HardHits.Count -gt 0) {
  Write-Host "HARD legacy scan FAILED ($($HardHits.Count) lines) — see $HardReport"
  exit 1
}

if ($SoftHits.Count -eq 0) {
  Write-Host "Legacy scan CLEAN (soft+hard)"
  exit 0
} else {
  Write-Host "Soft legacy scan found $($SoftHits.Count) hit lines — see $Report (hard CLEAN)"
  exit 0
}
