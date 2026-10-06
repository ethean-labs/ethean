# Local multi-node proving devnet on one Windows host (no Docker).
#
# Builds ethean + ethean-prover (release), writes a bundle with real PROD XMSS
# keys via `ethean devnet-init` (keys are generated once and reused), starts
# every node, polls each node's /lean/v0/fork_choice, then stops them.
#
# Optional scenarios (seconds are counted from genesis):
#   -LateJoin k -LateJoinAt s   node k starts only at s, via --checkpoint-sync-url
#                               against node 0
#   -Restart k -RestartAt s -RestartDown d
#                               node k is killed at s and started again d seconds
#                               later from its own data dir (no --reset-chain)
#
# Usage (repo root):
#   .\scripts\local-devnet.ps1
#   .\scripts\local-devnet.ps1 -Nodes 4 -RunSeconds 300 -MaxBlockData 3 -DebugLog
#   .\scripts\local-devnet.ps1 -Nodes 4 -LateJoin 3 -LateJoinAt 90 -Restart 2 -RestartAt 150
#   .\scripts\local-devnet.ps1 -Nodes 6 -ValidatorsPerNode 2 -Subnets 2 -Aggregators 2
#
# -Subnets sets ATTESTATION_COMMITTEE_COUNT; node k owns validators of subnet
# k % Subnets, so aggregators 0..Subnets-1 each prove one subnet.
param(
    [int]$Nodes = 3,
    [int]$ValidatorsPerNode = 1,
    [int]$Subnets = 1,
    [int]$Aggregators = 1,
    [int]$GenesisDelay = 30,
    [int]$RunSeconds = 180,
    [int]$PollSeconds = 8,
    [int]$MaxBlockData = -1,
    [switch]$DebugLog,
    [int]$LateJoin = -1,
    [int]$LateJoinAt = 60,
    [int]$Restart = -1,
    [int]$RestartAt = 120,
    [int]$RestartDown = 20,
    [string]$Out = "target/local-devnet"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

if ($LateJoin -eq 0 -or $Restart -eq 0) { throw "node 0 serves checkpoints and aggregates; pick another node" }
if ($LateJoin -ge $Nodes -or $Restart -ge $Nodes) { throw "scenario node index must be below -Nodes" }

Write-Host "==> Building ethean and ethean-prover (release)"
cargo build --release -p ethean -p ethean-prover
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
$Ethean = Join-Path $Root "target\release\ethean.exe"
$env:ETHEAN_PROVER_BIN = Join-Path $Root "target\release\ethean-prover.exe"
# One prover per node. Each one takes an equal share of the cores, so they
# do not all size their thread pools from the full core count.
$env:ETHEAN_PROVER_COUNT = "$Nodes"

Write-Host "==> Writing devnet bundle ($Nodes nodes x $ValidatorsPerNode validators, $Subnets subnets, $Aggregators aggregators) under $Out"
$init = & $Ethean devnet-init --out $Out --nodes $Nodes --genesis-delay $GenesisDelay `
    --validators-per-node $ValidatorsPerNode --attestation-committee-count $Subnets --aggregators $Aggregators
if ($LASTEXITCODE -ne 0) { throw "devnet-init failed" }
$init | Where-Object { $_ -notmatch "^ethean start" } | ForEach-Object { Write-Host $_ }
$commands = @($init | Where-Object { $_ -match "^ethean start" })
if ($commands.Count -ne $Nodes) { throw "expected $Nodes start commands, got $($commands.Count)" }
$genesisAt = (Get-Date).AddSeconds($GenesisDelay)

$logDir = Join-Path $Out "logs"
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

function Get-ArgLine([int]$k, [string]$extra) {
    $line = $commands[$k].Substring("ethean ".Length) + " --no-banner"
    if ($MaxBlockData -ge 0) { $line += " --max-block-attestation-data $MaxBlockData" }
    if ($DebugLog) { $line += " -v" }
    return $line + $extra
}

function Start-Node([int]$k, [string]$argLine, [string]$tag) {
    $p = Start-Process -FilePath $Ethean -ArgumentList $argLine -PassThru -NoNewWindow `
        -RedirectStandardOutput (Join-Path $logDir "ethean_$k$tag.out.log") `
        -RedirectStandardError (Join-Path $logDir "ethean_$k$tag.err.log")
    Write-Host "    started ethean_$k$tag (pid $($p.Id))"
    return $p
}

$procs = @{}
for ($k = 0; $k -lt $Nodes; $k++) {
    if ($k -eq $LateJoin) { continue }
    $procs[$k] = Start-Node $k (Get-ArgLine $k "") ""
}

function Get-View([int]$k) {
    $port = 5052 + $k
    try {
        $fc = Invoke-RestMethod -TimeoutSec 2 "http://127.0.0.1:$port/lean/v0/fork_choice"
        $head = $fc.nodes | Where-Object { $_.root -eq $fc.head } | Select-Object -First 1
        $safe = $fc.nodes | Where-Object { $_.root -eq $fc.safe_target } | Select-Object -First 1
        $headSlot = if ($head) { $head.slot } else { "?" }
        $safeSlot = if ($safe) { $safe.slot } else { "?" }
        return "h=$headSlot s=$safeSlot j=$($fc.justified.slot) f=$($fc.finalized.slot)"
    } catch {
        return "down"
    }
}

$lateJoined = $false
$restartState = "pending"
try {
    $deadline = $genesisAt.AddSeconds($RunSeconds)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Seconds $PollSeconds
        $sinceGenesis = ((Get-Date) - $genesisAt).TotalSeconds
        if ($LateJoin -gt 0 -and -not $lateJoined -and $sinceGenesis -ge $LateJoinAt) {
            Write-Host "==> ethean_$LateJoin joins from node 0's finalized checkpoint"
            $procs[$LateJoin] = Start-Node $LateJoin (Get-ArgLine $LateJoin " --checkpoint-sync-url http://127.0.0.1:5052") ""
            $lateJoined = $true
        }
        if ($Restart -gt 0 -and $restartState -eq "pending" -and $sinceGenesis -ge $RestartAt) {
            Write-Host "==> killing ethean_$Restart"
            Stop-Process -Id $procs[$Restart].Id -Force
            $restartState = "down"
        }
        if ($Restart -gt 0 -and $restartState -eq "down" -and $sinceGenesis -ge ($RestartAt + $RestartDown)) {
            Write-Host "==> restarting ethean_$Restart from its data dir"
            $resume = (Get-ArgLine $Restart "") -replace " --reset-chain", ""
            $procs[$Restart] = Start-Node $Restart $resume ".restart"
            $restartState = "up"
        }
        $views = for ($k = 0; $k -lt $Nodes; $k++) { "ethean_${k}: $(Get-View $k)" }
        Write-Host ("[{0:HH:mm:ss}] {1}" -f (Get-Date), ($views -join " | "))
        foreach ($k in @($procs.Keys)) {
            $p = $procs[$k]
            if ($p.HasExited -and -not ($k -eq $Restart -and $restartState -eq "down")) {
                Write-Host "    ethean_$k (pid $($p.Id)) exited ($($p.ExitCode)); see $logDir"
            }
        }
    }
    for ($k = 0; $k -lt $Nodes; $k++) {
        $port = 5052 + $k
        try {
            Invoke-WebRequest -UseBasicParsing -TimeoutSec 5 -OutFile (Join-Path $logDir "fork_choice_$k.json") `
                "http://127.0.0.1:$port/lean/v0/fork_choice" | Out-Null
        } catch {
            Write-Host "    fork_choice dump failed for ethean_$k"
        }
    }
} finally {
    Write-Host "==> Stopping nodes"
    foreach ($p in $procs.Values) {
        if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
    }
    Write-Host "logs: $logDir"
}
