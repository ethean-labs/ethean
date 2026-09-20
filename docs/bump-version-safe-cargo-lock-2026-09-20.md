# Safe Cargo.lock version bump in bump-version scripts (2026-09-20)

## Problem

A blanket `0.1.N` → `0.1.(N+1)` replace in `Cargo.lock` also rewrote
`tracing-attributes` (and any other crate on the same patch), breaking
`cargo check`.

## What landed

| Script | Change |
| --- | --- |
| `scripts/bump-version.ps1` | Regex limited to `[[package]]` / `name = "ethean…"` stanzas; UTF-8 no BOM |
| `scripts/bump-version.sh` | Same via embedded Python |
| `docs/versioning.md` | Documents the ethean-only lock rule |

## Recipe

```powershell
.\scripts\bump-version.ps1
# commit VERSION + Cargo.toml + Cargo.lock
```
