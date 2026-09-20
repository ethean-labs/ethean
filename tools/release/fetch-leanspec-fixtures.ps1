# Download and verify leanSpec production-scheme fixtures (local / CI cache).
# Does not commit the archive. Uses spec/fixtures/phase-00/manifest.toml digests.

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root

$Manifest = Join-Path $Root "spec\fixtures\phase-00\manifest.toml"
$CacheRoot = Join-Path $Root ".cache\leanspec-fixtures"
$Archive = Join-Path $CacheRoot "fixtures-prod-scheme.tar.gz"
$Extracted = Join-Path $CacheRoot "extracted"

function Get-TomlValue([string]$text, [string]$key) {
  if ($text -match "(?m)^\s*$key\s*=\s*`"([^`"]+)`"") {
    return $Matches[1]
  }
  if ($text -match "(?m)^\s*$key\s*=\s*(\d+)\s*$") {
    return $Matches[1]
  }
  return $null
}

if (-not (Test-Path $Manifest)) {
  Write-Error "Missing manifest: $Manifest"
}

$toml = Get-Content -Raw $Manifest
$url = Get-TomlValue $toml "browser_download_url"
$wantSha = (Get-TomlValue $toml "sha256").ToLowerInvariant()
$wantSize = [int64](Get-TomlValue $toml "size_bytes")

if (-not $url -or -not $wantSha -or -not $wantSize) {
  Write-Error "manifest.toml missing browser_download_url / sha256 / size_bytes"
}

New-Item -ItemType Directory -Force -Path $CacheRoot | Out-Null

$needDownload = $true
if (Test-Path $Archive) {
  $len = (Get-Item $Archive).Length
  if ($len -eq $wantSize) {
    $have = (Get-FileHash -Path $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($have -eq $wantSha) {
      $needDownload = $false
      Write-Host "Archive already verified at $Archive"
    }
  }
}

if ($needDownload) {
  # Prefer a local research copy when present (gitignored).
  $local = Join-Path $Root "bazalinacaklar\fixtures-prod-scheme.tar.gz"
  if (Test-Path $local) {
    Write-Host "Copying local archive $local"
    Copy-Item -Force $local $Archive
  } else {
    Write-Host "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $Archive
  }
  $len = (Get-Item $Archive).Length
  if ($len -ne $wantSize) {
    Write-Error "size mismatch: got $len want $wantSize"
  }
  $have = (Get-FileHash -Path $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($have -ne $wantSha) {
    Write-Error "sha256 mismatch: got $have want $wantSha"
  }
  Write-Host "Verified sha256 $wantSha ($wantSize bytes)"
}

if (Test-Path $Extracted) {
  Remove-Item -Recurse -Force $Extracted
}
New-Item -ItemType Directory -Force -Path $Extracted | Out-Null
Write-Host "Extracting to $Extracted"
tar -xzf $Archive -C $Extracted

$consensus = Join-Path $Extracted "fixtures\consensus"
if (-not (Test-Path $consensus)) {
  Write-Error "expected fixtures/consensus under extract root"
}

Write-Host ""
Write-Host "Ready. Point consumers at:"
Write-Host "  `$env:ETHEAN_LEANSPEC_FIXTURES = `"$Extracted`""
Write-Host "Then: cargo test -p ethean-spec-fixtures"
Write-Host "Hive matrix still needs an ethereum/hive lean client image for Ethean (see docs)."
