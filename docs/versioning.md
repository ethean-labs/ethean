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

The scripts update `VERSION`, root `Cargo.toml`, and **only** `ethean*` package
stanzas in `Cargo.lock`. Do not blanket-replace version strings in the lockfile
(that can overwrite crates.io pins such as `tracing-attributes`).

Then commit `VERSION` + `Cargo.toml` + `Cargo.lock`.

## Changelog and Releases

After a finished development update, append a short bullet under
`## [Unreleased]` in [`CHANGELOG.md`](../CHANGELOG.md) when the change is
operator-visible (API, CLI, storage schema, networking, crypto gates,
metrics, or breaking behaviour). Skip pure docs/process noise.

When cutting a public milestone:

1. Move `[Unreleased]` items into a new `## [X.Y.Z] - YYYY-MM-DD` section.
2. Add compare links at the bottom of `CHANGELOG.md`.
3. Create an annotated tag `vX.Y.Z` on the release commit and a notes-only
   GitHub Release whose body matches that section.
4. Leave binary / SBOM attachment to the repro path under `tools/release/`
   when promotion gates require it.
