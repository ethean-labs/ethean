# Migration protocol and retirement documentation — 2026-09-19

## Summary

Added and expanded English planning documents under `road-to/lean-consensus-migration/` for protocol open decisions and repository retirement. No product code was changed.

## Files written or updated

| File | Action |
| --- | --- |
| `02-protocol/OPEN_SPEC_DECISIONS.md` | Rewritten — nine blocking decisions (MAX_ATTESTATION_DATA, key/signature sizes, leanSig vs leanVM, Type-1/2 proofs, 3SF vs Goldfish, Snappy, fixture SHA, leanMetrics) with peer commits, authority, blocked phases |
| `05-retirement/DELETION_REGISTER.md` | Expanded — full inventory for `src/*`, `panro` Cargo.toml, examples, old `road-to/` docs, empty-dir rule |
| `05-retirement/REQUIRED_DIRECTORY_POLICY.md` | Expanded — `deploy/`, `deploy/observability/`, observability-as-needed policy |
| `05-retirement/LEGACY_NAME_ALLOWLIST.md` | **New** — allowlisted paths and `rg` search gates |

## Unchanged

- `02-protocol/UPSTREAM_REFRESH_POLICY.md` — not modified
- `05-retirement/README.md` — already linked the new allowlist

## References

- `bazalinacaklar/peer-client-library/protocol-surface-checklist.md`
- `02-protocol/COMPATIBILITY_LEDGER.md`

## Gate status

All listed protocol decisions remain **`status: unresolved`**. P0 must close them before consensus-facing implementation.
