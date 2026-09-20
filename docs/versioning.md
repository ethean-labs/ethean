# Version file

Single source of truth for the Ethean workspace version (`MAJOR.MINOR.PATCH`).

- Tracked in git as `VERSION` (this folder's parent file).
- Synced into `[workspace.package] version` in root `Cargo.toml`.
- Patch increments on each development update: `0.1.0` → `0.1.1` → … → `0.1.99` → `0.2.0`.
- When patch would exceed `99`, minor bumps and patch resets to `0`.
- When minor would exceed `99`, major bumps and minor/patch reset to `0`.

Bump with:

```powershell
.\scripts\bump-version.ps1
```

```bash
./scripts/bump-version.sh
```

Then commit `VERSION` + `Cargo.toml` (+ `Cargo.lock` if cargo rewrites it).
