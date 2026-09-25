# Lean Consensus migration planning library — completion

Date: 2026-09-19  
Branch: `plan/lean-consensus-migration`  
Planning baseline: `880982f`

## What landed

The full replacement planning library is under `road-to/lean-consensus-migration/`:

- `00-charter/` — mission, scope, success criteria
- `01-baseline/` — current-state audit, legacy matrix, removal map
- `02-protocol/` — authority, compatibility ledger, upstream refresh, open decisions
- `03-architecture/` — target workspace, dependency rules, data flow, 2000-line policy
- `04-risks/` — risk register, security gates, performance budgets, data migration
- `05-retirement/` — deletion register, required directories, legacy-name allowlist
- `06-observability/` — metrics contract, Prometheus topology, Grafana, alerts/SLOs, multinode analysis
- `phases/` — executable plans `00` through `13` with pin, gate, fixture, and deletion exits

`road-to/README.md` now points only at this library. Superseded Panro/Beacon roadmap documents under `road-to/` were deleted after transferable facts were recorded in baseline and retirement docs. Local Turkish draft `road-to/notlar.txt` was deleted after its intent was absorbed into the English charter.

`examples/README.md` was recreated to document removal of the Beacon `integration_example.rs` and the phase schedule for new Lean examples. Product `src/` code was not modified in this planning task.

## Authority reminder

- `leanSpec` pin + fixtures beat peer majority.
- Unresolved items in `OPEN_SPEC_DECISIONS.md` and `COMPATIBILITY_LEDGER.md` block the phases that consume them.
- Peer evidence commits remain those recorded in the peer research library (`docs/lean-peer-client-research-library-2026-09-19.md`).

## Next work (implementation, not this commit)

Start Phase 00 only after accepting the planning library: freeze the compatibility ledger, then follow the critical path in `phases/README.md`.
