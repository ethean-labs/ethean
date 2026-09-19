# Start Prometheus + Grafana for Ethean long-run metrics (Windows).
# Requires a healthy Docker Desktop Linux engine (Hyper-V / Virtual Machine Platform).
$ErrorActionPreference = "Stop"

function Get-DockerHint([string]$text) {
    if ($text -match 'dockerDesktopLinuxEngine|_ping|500 Internal Server Error|requested API version|HCS_E_HYPERV|HypervisorPresent|timeout waiting') {
        return @"

Docker Desktop UI can be open while the Linux engine is down (HTTP 500 on
dockerDesktopLinuxEngine/_ping). On this host that usually means Hyper-V /
Virtual Machine Platform is off (HCS_E_HYPERV_NOT_INSTALLED).

Fix (elevated PowerShell), then reboot:
  wsl.exe --install --no-distribution
  dism.exe /Online /Enable-Feature /FeatureName:VirtualMachinePlatform /All /NoRestart

Also enable CPU virtualization in firmware if Get-CimInstance Win32_ComputerSystem
shows HypervisorPresent = False.

Until then, skip Grafana and scrape the node:
  ethean start --until-signal --network pq-devnet-4
  curl http://127.0.0.1:9100/metrics
"@
    }
    if ($text -match 'wsl|WSL|Sistem dosyaya') {
        return @"

This looks like a Docker Desktop / WSL failure (wsl.exe exit 1).
Fix on Windows:
  1. Quit Docker Desktop fully
  2. In admin PowerShell: wsl --shutdown
  3. Optional: wsl --update
  4. Start Docker Desktop; wait until Engine is running
  5. Re-run: .\scripts\run-observability.ps1

Ethean does NOT need Grafana to advance head — use:
  ethean start --until-signal --network pq-devnet-4
  curl http://127.0.0.1:9100/metrics
"@
    }
    return @"

Start Docker Desktop, wait until it says Engine running, then re-run:
  .\scripts\run-observability.ps1
"@
}

function Invoke-DockerInfo {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = "docker"
    $psi.Arguments = "info"
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    if (-not $p.WaitForExit(15000)) {
        try { $p.Kill() } catch { }
        return @{ Exit = -1; Text = "timeout waiting for docker info" }
    }
    $out = $p.StandardOutput.ReadToEnd() + $p.StandardError.ReadToEnd()
    return @{ Exit = $p.ExitCode; Text = $out }
}

function Assert-DockerReady {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw @"
Docker CLI not found in PATH.
Install Docker Desktop, then reopen this terminal.

Until Docker works, run ethean alone and scrape:
  curl http://127.0.0.1:9100/metrics
  curl http://127.0.0.1:9100/healthz
"@
    }
    $info = Invoke-DockerInfo
    if ($info.Exit -ne 0) {
        throw "Docker daemon not ready.$($info.Text)`n$(Get-DockerHint $info.Text)"
    }
}

Assert-DockerReady
Set-Location (Join-Path $PSScriptRoot "..\deploy\observability")
$prev = $ErrorActionPreference
$ErrorActionPreference = "Continue"
$composeOut = & docker compose up -d 2>&1 | Out-String
$composeExit = $LASTEXITCODE
$ErrorActionPreference = $prev
if ($composeExit -ne 0) {
    throw @"
docker compose up -d failed (exit $composeExit).
$composeOut
$(Get-DockerHint $composeOut)
Ethean metrics still work at :9100 without Grafana.
"@
}

Write-Host "Waiting for Prometheus..."
$ok = $false
for ($i = 0; $i -lt 30; $i++) {
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:9090/-/ready" -UseBasicParsing -TimeoutSec 2
        if ($r.StatusCode -eq 200) { $ok = $true; break }
    } catch { Start-Sleep -Seconds 1 }
}
if (-not $ok) {
    Write-Warning "Prometheus not ready yet; open http://localhost:9090 after Docker finishes pulling."
}
Write-Host "Grafana: http://localhost:3000  Prometheus: http://localhost:9090"
Write-Host "Scrape target expects ethean on host :9100 (ethean start --until-signal)"
