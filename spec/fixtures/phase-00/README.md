# Phase 00 fixtures

Provenance only. The production fixture archive is **not** stored in git.

## Pin

| Field | Value |
| --- | --- |
| Release name | Latest production fixtures |
| Release tag (locator) | `latest` |
| Asset | `fixtures-prod-scheme.tar.gz` |
| Size | 154123448 bytes |
| SHA-256 | `21d9de7056b4e658031dc09e50c0e9dc1b0206089253258ce69ecff01154d4bd` |
| Generator | `leanSpec@0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8` |

Machine-readable record: [`manifest.toml`](./manifest.toml).

## Retrieval (operator / CI)

```powershell
# Example only — always verify digest before extraction
$url = "https://github.com/leanEthereum/leanSpec/releases/download/latest/fixtures-prod-scheme.tar.gz"
# Download to a local cache path outside the git tree, then:
# Get-FileHash <path> -Algorithm SHA256
# Compare to manifest.toml sha256 and size_bytes
```

Do not vendor the tarball under `spec/fixtures/`.
