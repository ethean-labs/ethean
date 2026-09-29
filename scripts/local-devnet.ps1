# Local multi-node proving devnet on one Windows host (no Docker).
#
# Builds ethean + ethean-prover (release), writes a bundle with real PROD XMSS
# keys via `ethean devnet-init` (keys are generated once and reused), starts
# every node, polls each node's /lean/v0/fork_choice, then stops them.
#
# Usage (repo root):
#   .\scripts\local-devnet.ps1
#   .\scripts\local-devnet.ps1 -Nodes 4 -RunSeconds 300 -MaxBlockData 3 -DebugLog
param(
    [int]$Nodes = 3,
    [int]$GenesisDelay = 30,
    [int]$RunSeconds = 180,
    [int]$PollSeconds = 8,
    [int]$MaxBlockData = -1,
    [switch]$DebugLog,
    [string]$Out = "target/local-devnet"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Host "==> Building ethean and ethean-prover (release)"
cargo build --release -p ethean -p ethean-prover
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
$Ethean = Join-Path $Root "target\release\ethean.exe"
$env:ETHEAN_PROVER_BIN = Join-Path $Root "target\release\ethean-prover.exe"

Write-Host "==> Writing devnet bundle ($Nodes nodes) under $Out"
$init = & $Ethean devnet-init --out $Out --nodes $Nodes --genesis-delay $GenesisDelay
if ($LASTEXITCODE -ne 0) { throw "devnet-init failed" }
$init | Where-Object { $_ -notmatch "^ethean start" } | ForEach-Object { Write-Host $_ }
$commands = @($init | Where-Object { $_ -match "^ethean start" })
if ($commands.Count -ne $Nodes) { throw "expected $Nodes start commands, got $($commands.Count)" }

$logDir = Join-Path $Out "logs"
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$procs = @()
for ($k = 0; $k -lt $Nodes; $k++) {
    $argLine = $commands[$k].Substring("ethean ".Length) + " --no-banner"
    if ($MaxBlockData -ge 0) { $argLine += " --max-block-attestation-data $MaxBlockData" }
    if ($DebugLog) { $argLine += " -v" }
    $procs += Start-Process -FilePath $Ethean -ArgumentList $argLine -PassThru -NoNewWindow `
        -RedirectStandardOutput (Join-Path $logDir "ethean_$k.out.log") `
        -RedirectStandardError (Join-Path $logDir "ethean_$k.err.log")
    Write-Host "    started ethean_$k (pid $($procs[-1].Id))"
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

try {
    $deadline = (Get-Date).AddSeconds($GenesisDelay + $RunSeconds)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Seconds $PollSeconds
        $views = for ($k = 0; $k -lt $Nodes; $k++) { "ethean_${k}: $(Get-View $k)" }
        Write-Host ("[{0:HH:mm:ss}] {1}" -f (Get-Date), ($views -join " | "))
        foreach ($p in $procs) {
            if ($p.HasExited) { Write-Host "    pid $($p.Id) exited ($($p.ExitCode)); see $logDir" }
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
    foreach ($p in $procs) {
        if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
    }
    Write-Host "logs: $logDir"
}
