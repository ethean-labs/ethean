# Package the ethean binary for one rustc target into dist/.
# Usage: pwsh tools/release/package-ethean.ps1 -Version 0.1.47 -Target x86_64-pc-windows-msvc
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Target,
    [string]$DistDir = ""
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
if (-not $DistDir) {
    $DistDir = Join-Path $Root "dist"
}
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

$isWindowsTarget = $Target -like "*-pc-windows-*"
$binName = if ($isWindowsTarget) { "ethean.exe" } else { "ethean" }
$candidates = @(
    (Join-Path $Root "target\$Target\release\$binName"),
    (Join-Path $Root "target\release\$binName")
)
$bin = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $bin) {
    throw "binary not found for target=$Target (looked: $($candidates -join ', '))"
}

$stage = Join-Path ([System.IO.Path]::GetTempPath()) ("ethean-pkg-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Force -Path $stage | Out-Null
try {
    Copy-Item -Force $bin (Join-Path $stage $binName)
    $base = "ethean-v$Version-$Target"
    if ($isWindowsTarget) {
        $archive = Join-Path $DistDir "$base.zip"
        if (Test-Path $archive) { Remove-Item -Force $archive }
        Compress-Archive -Path (Join-Path $stage $binName) -DestinationPath $archive -Force
    } else {
        $archive = Join-Path $DistDir "$base.tar.gz"
        tar -czf $archive -C $stage $binName
    }

    $hash = (Get-FileHash -Algorithm SHA256 -Path $archive).Hash.ToLowerInvariant()
    $shaFile = Join-Path $DistDir "$base.sha256"
    Set-Content -Encoding ascii -Path $shaFile -Value "$hash  $(Split-Path -Leaf $archive)"
    Write-Host "wrote $archive"
    Write-Host "wrote $shaFile"
}
finally {
    Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
}
