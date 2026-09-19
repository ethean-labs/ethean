# Lean consensus migration — phases 03–06 planning update (2026-09-19)

## Summary

Added and refreshed English phase plan documents for the protocol foundation segment of the Ethean Lean Consensus migration. No product Rust code was changed.

## Files created or updated

| File | Action |
| --- | --- |
| `road-to/lean-consensus-migration/phases/03-canonical-ssz-and-types.md` | Rewritten to match phase 00/01/07 structure |
| `road-to/lean-consensus-migration/phases/04-genesis-presets-and-clock.md` | Rewritten with 4s slot and Beacon timing deletion obligations |
| `road-to/lean-consensus-migration/phases/05-state-transition.md` | Rewritten as pure transactional Lean STF |
| `road-to/lean-consensus-migration/phases/06-fork-choice-and-finality.md` | **New** — 3SF-mini or pinned Goldfish only |
| `road-to/lean-consensus-migration/phases/README.md` | Updated index for phases 00–13 and critical path |

## Shared conventions applied

- Required headings: Pinned inputs, Objective, Non-goals, Entry criteria, Exact old and new paths, Ordered tasks, Deletion obligations, Security/spec risks, Positive and negative fixtures, Interop and differential tests, Validation commands, Exit criteria, Rollback/data policy, Artifacts/evidence, Dependencies.
- Pinned evidence pattern: baseline `880982f`, leanSpec candidates `8b4ebbea` / `0b7d33ec` (Phase 00 verification required), seven peer commits, unresolved values marked as blockers.
- `MAX_ATTESTATION_DATA` must be resolved in Phase 00 before Phase 03 starts.
- Hand-written source files capped at 300 lines per repository modularity policy.
- Crate paths aligned with [TARGET_WORKSPACE](../road-to/lean-consensus-migration/03-architecture/TARGET_WORKSPACE.md) (`crates/types`, `crates/ethean-ssz`, `crates/ethean-fork-choice`, etc.).
- `bazalinacaklar/` noted as present locally with peer library notes (corrects prior README claim of absence).

## Critical path (unchanged intent)

`00 → 01 → 02 → 03 → 04 → 05 → 06` → signer/aggregation/duties (07–09) → network/storage (10–11) → interop/release (12–13).

## Planned follow-ups

- Phase 12 (`12-api-observability-and-multiclient-interop.md`) and Phase 13 (`13-security-performance-and-release.md`) already exist as separate documents; the phases README now links all phases 00–13.
- Phase 00 must close OSD-001 (`MAX_ATTESTATION_DATA`) and OSD-006 (fork-choice generation) before phases 03 and 06 implementation begins.
