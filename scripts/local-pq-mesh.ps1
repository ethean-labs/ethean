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
    [int]$WaitDialableSec = 45,
    [int]$Validators = 4
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

Write-Host "==> Starting peer A (--until-signal --network $Network --validators $Validators)…"
$env:RUST_LOG = "info"
$procA = Start-Process -FilePath $Ethean `
    -ArgumentList @("start", "--until-signal", "--network", $Network, "--validators", "$Validators") `
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

# Prefer loopback so peer B on the same host can dial.
$dialLoopback = $dialable -replace '/ip4/[^/]+/', '/ip4/127.0.0.1/'
Set-Content -Path $NodesFile -Value $dialLoopback -NoNewline
Write-Host "==> Wrote $NodesFile → $dialLoopback"

Write-Host "==> Starting peer B (--ticks $PeerBTicks --wall-clock --bootnodes …)…"
$procB = Start-Process -FilePath $Ethean `
    -ArgumentList @(
        "start", "--ticks", "$PeerBTicks", "--wall-clock",
        "--network", $Network, "--validators", "$Validators",
        "--bootnodes", $dialLoopback
    ) `
    -WorkingDirectory $Root `
    -RedirectStandardOutput $LogB `
    -RedirectStandardError $ErrB `
    -PassThru `
    -NoNewWindow `
    -Wait

Write-Host "==> Peer B exit code $($procB.ExitCode)"
Write-Host "==> Logs: $LogA / $ErrA / $LogB / $ErrB"
Stop-Process -Id $procA.Id -Force -ErrorAction SilentlyContinue
if ($procB.ExitCode -ne 0) { exit $procB.ExitCode }
