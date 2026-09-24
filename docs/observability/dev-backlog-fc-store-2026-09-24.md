# Development backlog and next-step decision (2026-09-24)

## Current snapshot (v0.1.51)

- Live structural `ForkChoiceStore` on `ChainOwner` (head / safe-target / reorg)
- Known-parent import + durable rebuild on `--data-dir` resume
- Verified gossip and local attestations feed the store pending pool

## Backlog (priority)

| Pri | Area | Action |
| --- | --- | --- |
| P0–P1 | Live FC + head + resume + vote ingest | **Done** |
| P1 | Fixture re-fill | Upstream empty-body `at_9` / `dead_9` (watch only) |
| P2 | Operator A2/A3 / Hive | External pins only; no invented digests |
| P2 | leanBench alignment | **Done** — pin table in `docs/lean-crypto/leanbench-alignment-notes-2026-09-24.md` |

## This session slice

1. Wire `fc_on_attestation` / `fc_on_aggregated` after verify — done.
2. Version bump to **0.1.51**.
