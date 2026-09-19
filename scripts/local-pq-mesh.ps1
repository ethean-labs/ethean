# Local private PQ mesh (2 peers) — Windows PowerShell
#
# Ream-style flow: start peer A, write its dialable multiaddr as the mesh
# "nodes" list, then peer B dials that file (Ethean equivalent of nodes.yaml).
#
# Usage (from repo root):
#   .\scripts\local-pq-mesh.ps1
#   .\scripts\local-pq-mesh.ps1 -Network pq-devnet-4 -PeerBTicks 15
#
# Stops peer A when the script exits (Ctrl-C or completion).

param(
    [string]$Network = "pq-devnet-4",
    [int]$PeerBTicks = 12,
    [int]$WaitDialableSec = 45
)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root

$MeshDir = Join-Path $Root "target\local-pq-mesh"
New-Item -ItemType Directory -Force -Path $MeshDir | Out-Null
$LogA = Join-Path $MeshDir "peer-a.log"
$LogB = Join-Path $MeshDir "peer-b.log"
$NodesFile = Join-Path $MeshDir "nodes.multiaddrs"
$ErrA = Join-Path $MeshDir "peer-a.err"
$ErrB = Join-Path $MeshDir "peer-b.err"

function Find-Ethean {
    $candidates = @(
        (Join-Path $Root "target\release\ethean.exe"),
        (Join-Path $Root "target\debug\ethean.exe")
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return $c }
    }
    $cmd = Get-Command ethean -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

Write-Host "==> Building ethean (release)…"
cargo build -p ethean --release
$Ethean = Find-Ethean
if (-not $Ethean) { throw "ethean binary not found after build" }
Write-Host "==> Using $Ethean"

Remove-Item $LogA, $LogB, $ErrA, $ErrB, $NodesFile -ErrorAction SilentlyContinue

Write-Host "==> Starting peer A (--until-signal --network $Network)…"
$env:RUST_LOG = "info"
$procA = Start-Process -FilePath $Ethean `
    -ArgumentList @("start", "--until-signal", "--network", $Network) `
    -WorkingDirectory $Root `
    -RedirectStandardOutput $LogA `
    -RedirectStandardError $ErrA `
    -PassThru `
    -NoNewWindow

$dialable = $null
$deadline = (Get-Date).AddSeconds($WaitDialableSec)
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 400
    $text = ""
    if (Test-Path $LogA) { $text += Get-Content $LogA -Raw -ErrorAction SilentlyContinue }
    if (Test-Path $ErrA) { $text += Get-Content $ErrA -Raw -ErrorAction SilentlyContinue }
    if ($text -match 'dialable=([^\s]+)') {
        $dialable = $Matches[1]
        break
    }
    if ($procA.HasExited) {
        throw "peer A exited early (code $($procA.ExitCode)). See $LogA / $ErrA"
    }
}

if (-not $dialable) {
    Stop-Process -Id $procA.Id -Force -ErrorAction SilentlyContinue
    throw "timed out waiting for peer A dialable= (see $LogA / $ErrA)"
}

# Same-host dial: rewrite advertised LAN IP to loopback.
$boot = $dialable -replace '^/ip4/[^/]+/', '/ip4/127.0.0.1/'
@"
# Auto-generated local private mesh peer list (Ethean multiaddrs).
# Equivalent role to lean-quickstart / Ream genesis nodes.yaml for this run.
# Peer A dialable (original): $dialable
$boot
"@ | Set-Content -Path $NodesFile -Encoding utf8

Write-Host "==> Wrote mesh nodes file: $NodesFile"
Write-Host "    bootnode: $boot"

Write-Host "==> Starting peer B (dial mesh, --ticks $PeerBTicks --wall-clock)…"
$procB = Start-Process -FilePath $Ethean `
    -ArgumentList @(
        "start",
        "--network", $Network,
        "--ticks", "$PeerBTicks",
        "--wall-clock",
        "--bootnodes", $boot
    ) `
    -WorkingDirectory $Root `
    -RedirectStandardOutput $LogB `
    -RedirectStandardError $ErrB `
    -PassThru `
    -NoNewWindow

$procB.WaitForExit()
$exitB = $procB.ExitCode

Write-Host "==> Stopping peer A…"
Stop-Process -Id $procA.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

$logBText = ""
if (Test-Path $LogB) { $logBText += Get-Content $LogB -Raw -ErrorAction SilentlyContinue }
if (Test-Path $ErrB) { $logBText += Get-Content $ErrB -Raw -ErrorAction SilentlyContinue }
$logAText = ""
if (Test-Path $LogA) { $logAText += Get-Content $LogA -Raw -ErrorAction SilentlyContinue }
if (Test-Path $ErrA) { $logAText += Get-Content $ErrA -Raw -ErrorAction SilentlyContinue }

$dialOk = ($logBText -match 'dialed bootnode') -or ($logBText -match 'ConnectionEstablished')
$statusOk = ($logBText -match 'Status handshake') -or ($logAText -match 'Status handshake')

Write-Host ""
Write-Host "=== Local private mesh result ==="
Write-Host "nodes file : $NodesFile"
Write-Host "peer A log : $LogA / $ErrA"
Write-Host "peer B log : $LogB / $ErrB"
Write-Host "peer B exit: $exitB"
Write-Host "dial seen  : $dialOk"
Write-Host "status seen: $statusOk"

if ($exitB -ne 0) { exit $exitB }
if (-not $dialOk) {
    Write-Host "WARN: bootnode dial line not found; check peer B logs."
    exit 2
}
Write-Host "OK: private mesh dial path exercised."
exit 0
