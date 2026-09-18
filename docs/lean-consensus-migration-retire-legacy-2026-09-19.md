# Lean migration — planning retirement closeout

Date: 2026-09-19  
Branch: `plan/lean-consensus-migration`

## Planning retirement (complete)

Section 8 of the migration roadmap is closed for the planning library task:

- `road-to/` is a short index pointing at `lean-consensus-migration/` only.
- Superseded Panro/Beacon roadmap files (root manifesto/architecture/tech/week reports and `road-to/` copies) are deleted; Git history is the archive.
- `road-to/notlar.txt` is gone; intent lives in the English charter.
- `examples/integration_example.rs` is deleted; `examples/README.md` documents Lean example phases.
- Root `README.md` and `docs/README.md` link the migration library without rewriting product docs.

Deletion status is recorded in
[DELETION_REGISTER.md](../road-to/lean-consensus-migration/05-retirement/DELETION_REGISTER.md).

## Explicitly deferred (not this task)

- Product `src/` removal (including WOTS/BLS paths) waits for owning phases 01–13.
- Remaining `docs/` week/Panro/Beam duplicates wait for Phase 13 / R5 execution.
- No push of the planning branch was performed.
