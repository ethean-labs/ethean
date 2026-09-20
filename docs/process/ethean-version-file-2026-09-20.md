# Workspace VERSION file and bump scripts (2026-09-20)

Introduced root `VERSION` as the release number source, synced to
`[workspace.package] version` in `Cargo.toml`.

Bump rule: increment patch on each finished development update
(`0.1.0` → `0.1.1` → …). When patch would exceed `99`, minor++ and patch=0
(`0.1.99` → `0.2.0`). Same for minor→major at 99.

Helpers: `scripts/bump-version.ps1`, `scripts/bump-version.sh`.
First version after adopting the file: **0.1.1**.
