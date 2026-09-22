# Multi-platform ethean binary releases (2026-09-23)

## Why

Milestone GitHub Releases were notes-only. Operators asked for
Lighthouse-style platform archives (Linux / macOS / Windows, x86_64 and
aarch64) attached to releases. Windows is experimental until leanMultisig
compiles there (the job is `continue-on-error`).

## What landed

| Piece | Role |
| --- | --- |
| `tools/release/package-ethean.sh` | Archive `ethean` + `ethean-prover` and a `.sha256` with stable names |
| `tools/release/render-binaries-notes.sh` | Markdown Binaries table for release bodies |
| `.github/workflows/release-binaries.yml` | Matrix build + upload on `v*` tags / dispatch |
| `docs/release/binaries.md` | Operator doc for targets and features |

## Targets

- `x86_64-pc-windows-msvc` (`windows-latest`, experimental)
- `x86_64-unknown-linux-gnu` (`ubuntu-latest`)
- `aarch64-unknown-linux-gnu` (`ubuntu-latest` + `cross`)
- `aarch64-apple-darwin` / `x86_64-apple-darwin` (`macos-latest`)

## Follow-ups

- Optional PGP detached signatures (`.asc`) once a release signing key is published
- Docker Hub image row when an official image exists
- Attach the same matrix to future tags automatically on `git push --tags`
