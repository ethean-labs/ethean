# Soft legacy docs cleanup after node purge (2026-09-19)

- Replaced `docs/deployment.md` with Lean `ethean` smoke deploy notes.
- Deleted Turkish `docs/2026-09-19-proje-kod-incelemesi.md` (non-English + stale Beacon claims).
- `ethean-node` tests: 9 passed.
- Remaining soft-scan hits in older `docs/week*` Beacon notes — **deleted** in this pass; core guides (`architecture`, `consensus`, `networking`, `storage`, `security`, `monitoring`, `roadmap`, `deployment`) rewritten for Lean.
- Migration/planning docs may still mention `panro`/`blst` as historical evidence (allowlisted).
