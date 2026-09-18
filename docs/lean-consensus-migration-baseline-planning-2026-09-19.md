# Lean Consensus Migration — Baseline Planning Documents (2026-09-19)

## Summary

Added three English planning documents under `road-to/lean-consensus-migration/01-baseline/` to freeze the Beacon-era audit, path-level retirement matrix, and rewrite policy before any product code changes.

## Files

| Document | Purpose |
| --- | --- |
| `CURRENT_STATE_AUDIT.md` | Evidence-based audit of Panro/Ethean at commit `880982f`; peer commits; no-retain classification; planning boundary warning |
| `LEGACY_COMPONENT_MATRIX.md` | Exhaustive disposition matrix for roots, `src/`, `docs/`, examples, config/tests/benches/deployment gaps |
| `REMOVAL_REWRITE_REUSE_MAP.md` | Delete vs test-first redesign rules; forbids copying old `src/` |

Existing `01-baseline/README.md` already indexes these paths; links verified, not overwritten.

## Constraints honored

- No edits under `src/` or root `Cargo.toml`
- English only in tracked files
- Evidence from `bazalinacaklar/peer-client-library/ethean-current-baseline.md`

## Next steps

Phase 00 fixture pins and Phase 01 identity cleanup proceed against this baseline inventory.
