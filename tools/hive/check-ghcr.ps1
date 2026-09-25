# Probe whether ghcr.io/ethean-labs/ethean tags are anonymously pullable.
# Hive clients need a public (or otherwise pullable) :devnet5 base image.
param(
    [string]$Image = "ethean-labs/ethean",
    [string[]]$Tags = @("devnet5", "latest-devnet5", "unstable")
)

$ErrorActionPreference = "Continue"
$scope = "repository:${Image}:pull"
$tokenJson = curl.exe -sS "https://ghcr.io/token?service=ghcr.io&scope=$scope"
$token = $null
try {
    $token = ($tokenJson | ConvertFrom-Json).token
} catch {
    Write-Host "FAIL: anonymous GHCR token request failed"
    Write-Host $tokenJson
    exit 2
}

if (-not $token) {
    Write-Host "FAIL: package is private or missing (no anonymous pull token)"
    Write-Host $tokenJson
    Write-Host ""
    Write-Host "Fix: GitHub -> ethean-labs/ethean -> Packages -> ethean -> Package settings -> Change visibility -> Public"
    Write-Host "Or after gh auth:  gh api --method PUT -H 'Accept: application/vnd.github+json' /orgs/ethean-labs/packages/container/ethean/visibility -f visibility=public"
    exit 1
}

$ok = 0
foreach ($tag in $Tags) {
    $code = curl.exe -sS -o NUL -w "%{http_code}" `
        -H "Authorization: Bearer $token" `
        -H "Accept: application/vnd.oci.image.index.v1+json" `
        "https://ghcr.io/v2/${Image}/manifests/$tag"
    if ($code -eq "200") {
        Write-Host "OK   :$tag  (HTTP $code)"
        $ok++
    } else {
        Write-Host "MISS :$tag  (HTTP $code)"
    }
}

if ($ok -eq 0) {
    Write-Host ""
    Write-Host "No listed tags found. Publish with a v* tag (devnet5) or workflow_dispatch extra_tags=devnet5,latest-devnet5."
    exit 1
}

Write-Host ""
Write-Host "Pullable tag count: $ok / $($Tags.Count)"
exit 0
