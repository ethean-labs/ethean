# Bump Ethean workspace version (patch++ ; 0.1.99 -> 0.2.0)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$versionPath = Join-Path $root "VERSION"
$cargoPath = Join-Path $root "Cargo.toml"
$lockPath = Join-Path $root "Cargo.lock"

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
$prev = "$major.$minor.$patch"

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

if (Test-Path $lockPath) {
    # Only ethean* package stanzas — never blanket-replace (breaks crates.io pins).
    $bytes = [System.IO.File]::ReadAllBytes($lockPath)
    $offset = 0
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
        $offset = 3
    }
    $lock = [System.Text.Encoding]::UTF8.GetString($bytes, $offset, $bytes.Length - $offset)
    $lockPattern = "(?m)(^\[\[package\]\]\r?\nname = `"ethean[^`"]*`"\r?\n)version = `"$([regex]::Escape($prev))`""
    $lockUpdated = [regex]::Replace($lock, $lockPattern, "`${1}version = `"$next`"")
    $replaced = [regex]::Matches(
        $lockUpdated,
        "(?m)^name = `"ethean[^`"]*`"\r?\nversion = `"$([regex]::Escape($next))`""
    ).Count
    if ($replaced -eq 0) {
        throw "Cargo.lock has no ethean packages at $prev to bump"
    }
    if (-not $lockUpdated.StartsWith("# This file")) {
        throw "Cargo.lock rewrite lost the leading Cargo header comment"
    }
    $utf8 = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($lockPath, $lockUpdated, $utf8)
    Write-Host "Bumped version to $next (VERSION + Cargo.toml + Cargo.lock ethean* x$replaced)"
} else {
    Write-Host "Bumped version to $next (VERSION + Cargo.toml)"
}
