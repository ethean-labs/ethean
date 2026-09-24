param(
    [Parameter(Mandatory = $true)]
    [string]$HiveRoot
)

$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$dropin = Join-Path $repoRoot "docker\hive\upstream-clients-ethean"
$hive = Resolve-Path $HiveRoot
$dest = Join-Path $hive "clients\ethean"

if (-not (Test-Path $dropin)) {
    throw "missing drop-in at $dropin"
}

New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item -Force (Join-Path $dropin "Dockerfile") (Join-Path $dest "Dockerfile")
Copy-Item -Force (Join-Path $dropin "Dockerfile.git") (Join-Path $dest "Dockerfile.git")
Copy-Item -Force (Join-Path $dropin "ethean.sh") (Join-Path $dest "ethean.sh")
Copy-Item -Force (Join-Path $dropin "hive.yaml") (Join-Path $dest "hive.yaml")
Copy-Item -Force (Join-Path $dropin "validators.yaml") (Join-Path $dest "validators.yaml")
Write-Host "copied clients/ethean drop-in"

function Append-ClientYaml([string]$file, [string]$nametag) {
    $text = Get-Content -Raw $file
    if ($text -match '(?m)^- client: ethean\s*$') {
        Write-Host "ok: $file already lists ethean"
        return
    }
    if (-not $text.EndsWith("`n")) { $text += "`n" }
    $text += "`n- client: ethean`n  nametag: $nametag`n"
    Set-Content -Path $file -Value $text -NoNewline
    Write-Host "appended ethean to $file"
}

Append-ClientYaml (Join-Path $hive "simulators\lean\clients\devnet4.yaml") "devnet4"
Append-ClientYaml (Join-Path $hive "simulators\lean\clients\devnet5.yaml") "devnet5"

$matrix = Join-Path $hive "simulators\lean\config\lean-devnets.txt"
$matrixText = Get-Content -Raw $matrix
if ($matrixText -match '(?m)^ethean=devnet4,devnet5\s*$') {
    Write-Host "ok: lean-devnets.txt already lists ethean"
} else {
    if (-not $matrixText.EndsWith("`n")) { $matrixText += "`n" }
    $matrixText += "ethean=devnet4,devnet5`n"
    Set-Content -Path $matrix -Value $matrixText -NoNewline
    Write-Host "appended ethean matrix row to lean-devnets.txt"
}

$util = Join-Path $hive "simulators\lean\src\utils\util.rs"
$utilText = Get-Content -Raw $util
if ($utilText -match '"ethean"') {
    Write-Host "ok: util.rs already mentions ethean"
} elseif ($utilText.Contains('"ream",')) {
    $idx = $utilText.IndexOf('"ream",')
    $utilText = $utilText.Insert($idx + '"ream",'.Length, "`n            `"ethean`",")
    Set-Content -Path $util -Value $utilText -NoNewline
    Write-Host "patched util.rs lean client candidates"
} else {
    Write-Warning "util.rs layout unexpected; add `"ethean`" to lean_client_kind candidates manually"
}

$prep = Join-Path $hive "simulators\lean\helper\prepare_lean_client_assets.py"
$prepText = Get-Content -Raw $prep
if ($prepText -match '"ethean"') {
    Write-Host "ok: prepare_lean_client_assets.py already mentions ethean"
} else {
    $old = 'SUPPORTED_CLIENTS = {' + "`n" + '    "ethlambda",'
    $new = 'SUPPORTED_CLIENTS = {' + "`n" + '    "ethean",' + "`n" + '    "ethlambda",'
    if (-not $prepText.Contains($old)) {
        $old = 'SUPPORTED_CLIENTS = {' + "`n" + ' "ethlambda",'
        $new = 'SUPPORTED_CLIENTS = {' + "`n" + ' "ethean",' + "`n" + ' "ethlambda",'
    }
    if (-not $prepText.Contains($old)) {
        throw "prepare script: SUPPORTED_CLIENTS block not found"
    }
    $prepText = $prepText.Replace($old, $new)
    $prepText = $prepText.Replace('if CLIENT_KIND == "ream":', 'if CLIENT_KIND in {"ream", "ethean"}:')
    Set-Content -Path $prep -Value $prepText -NoNewline
    Write-Host "patched prepare_lean_client_assets.py"
}

Write-Host ""
Write-Host "Done. Next:"
Write-Host "  1. Ensure ghcr.io/ethean-labs/ethean:devnet5 exists (CI publish)."
Write-Host "  2. From hive root run the lean sim with --client ethean"
Write-Host "  3. Open ethereum/hive PR using tools/hive/PR_BODY.md"
