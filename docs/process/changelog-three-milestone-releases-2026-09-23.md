# Changelog and three milestone GitHub Releases (2026-09-23)

## Why

The repo tracked every development update under `docs/` and bumped
`VERSION` on each patch, but had no curated changelog and no GitHub
Releases. Operators had no single place to see what changed between
public cuts.

## What landed

| Piece | Detail |
| --- | --- |
| [`CHANGELOG.md`](../../CHANGELOG.md) | Keep a Changelog sections for `0.1.12`, `0.1.27`, `0.1.47` plus empty `[Unreleased]` |
| Tags | Annotated `v0.1.12` @ STF/FC green, `v0.1.27` @ operator readiness, `v0.1.47` @ tip (durable storage, FC, native XMSS) |
| GitHub Releases | Notes-only bodies matching the changelog sections |
| Docs wiring | Root README footer, `docs/README.md`, `docs/release/README.md`, `docs/versioning.md` |

## Cut rationale

Patch history from `0.1.1` to `0.1.47` is dense and same-day; three
feature arcs are clearer than forty-seven patch releases:

1. **0.1.12** — Lean workspace + leanSpec STF/FC fixture green
2. **0.1.27** — HTTP/QUIC, Hive, duties, leanVM/Type-1, operator readiness
3. **0.1.47** — Durable persist/prune, observability, FC finality/safe_target, native XMSS

## Follow-ups

- Append operator-visible bullets under `[Unreleased]` on each bump.
- Attach repro binaries / SBOM only when `docs/release/gates.md` promotion
  requires them.
