# Development backlog and next-step decision (2026-09-24)

## Current snapshot (v0.1.52)

- Live structural FC store: head / safe-target / reorg / vote ingest
- Local attest: head = FC tip, target = safe-target
- `GET /lean/v1/chain/fork_choice` operator view

## Backlog (priority)

| Pri | Area | Action |
| --- | --- | --- |
| P0–P1 | Live FC path | **Done** |
| P1 | Attest head≠target + FC HTTP | **Done** |
| P1 | Fixture re-fill | Upstream empty-body dumps (watch only) |
| P2 | Operator A2/A3 / Hive | External pins only; no invented digests |

## This session slice

1. Split attestation head/target — done.
2. Fork-choice Lean HTTP route — done.
3. Version bump to **0.1.52**.
