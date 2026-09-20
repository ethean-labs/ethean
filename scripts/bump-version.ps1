# Bump Ethean workspace version (patch++ ; 0.1.99 -> 0.2.0)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$versionPath = Join-Path $root "VERSION"
$cargoPath = Join-Path $root "Cargo.toml"

if (-not (Test-Path $versionPath)) {
    throw "VERSION file missing at $versionPath"
}

$raw = (Get-Content -Path $versionPath -Raw).Trim()
if ($raw -notmatch '^(\d+)\.(\d+)\.(\d+)$') {
    throw "VERSION must be MAJOR.MINOR.PATCH (got '$raw')"
}

$major = [int]$Matches[1]
$minor = [int]$Matches[2]
$patch = [int]$Matches[3]

$patch++
if ($patch -gt 99) {
    $patch = 0
    $minor++
    if ($minor -gt 99) {
        $minor = 0
        $major++
    }
}

$next = "$major.$minor.$patch"
Set-Content -Path $versionPath -Value "$next`n" -NoNewline

$cargo = Get-Content -Path $cargoPath -Raw
$updated = [regex]::Replace(
    $cargo,
    '(?m)^version = "[0-9]+\.[0-9]+\.[0-9]+"',
    "version = `"$next`"",
    1
)
if ($updated -eq $cargo) {
    throw "Could not update [workspace.package] version in Cargo.toml"
}
Set-Content -Path $cargoPath -Value $updated -NoNewline

Write-Host "Bumped version to $next (VERSION + Cargo.toml)"
