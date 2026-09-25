# Build a local ghcr.io/ethean-labs/ethean:devnet5 and run a narrow Hive sync smoke.
#
# Use when GHCR anonymous pull fails (private package) but Docker is available.
# Does not invent digests/bootnodes; does not open the ethereum/hive PR.
param(
    [string]$HiveRoot = "",
    [string]$Image = "ghcr.io/ethean-labs/ethean:devnet5",
    [switch]$SkipBuild,
    [switch]$SkipHiveRun,
    [string]$SimLimit = "sync"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Require-Docker {
    docker info 1>$null 2>$null
    if ($LASTEXITCODE -ne 0) {
        throw "Docker daemon is not running. Start Docker Desktop (or a Linux daemon) and retry."
    }
}

Require-Docker

if (-not $SkipBuild) {
    Write-Host "Building $Image from $RepoRoot ..."
    docker build -t $Image $RepoRoot
    if ($LASTEXITCODE -ne 0) { throw "docker build failed" }
    docker image inspect $Image 1>$null
    Write-Host "OK image $Image"
}

if (-not $HiveRoot) {
    $candidate = Join-Path $RepoRoot "bazalinacaklar\peer-repos\hive"
    if (Test-Path $candidate) { $HiveRoot = $candidate }
}

if (-not $HiveRoot -or -not (Test-Path $HiveRoot)) {
    Write-Host "HiveRoot not set. Apply manually:"
    Write-Host "  .\tools\hive\apply-ethean-client.ps1 -HiveRoot <path-to-hive>"
    Write-Host "Then: cd <hive>; go build -o hive.exe ."
    Write-Host "  .\hive.exe --sim lean --client ethean --client-file simulators/lean/clients/devnet5.yaml --sim.limit '$SimLimit'"
    exit 0
}

Write-Host "Applying Ethean client drop-in onto $HiveRoot ..."
& (Join-Path $PSScriptRoot "apply-ethean-client.ps1") -HiveRoot $HiveRoot

if ($SkipHiveRun) {
    Write-Host "SkipHiveRun set; apply done."
    exit 0
}

$hiveBin = Join-Path $HiveRoot "hive.exe"
if (-not (Test-Path $hiveBin)) {
    $hiveBin = Join-Path $HiveRoot "hive"
}
if (-not (Test-Path $hiveBin)) {
    Write-Host "No hive binary at $HiveRoot. Build it (go build -o hive.exe .) then re-run without -SkipHiveRun."
    exit 0
}

$results = Join-Path $HiveRoot "workspace\logs"
New-Item -ItemType Directory -Force -Path $results | Out-Null
Write-Host "Running Hive lean sim limit='$SimLimit' ..."
Push-Location $HiveRoot
try {
    & $hiveBin --sim lean --client ethean `
        --client-file simulators/lean/clients/devnet5.yaml `
        --sim.limit $SimLimit `
        --results-root $results
} finally {
    Pop-Location
}
