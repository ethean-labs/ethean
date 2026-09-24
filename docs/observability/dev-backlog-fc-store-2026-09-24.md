# Development backlog and next-step decision (2026-09-24)

## Current snapshot (v0.1.50)

- Optional live structural `ForkChoiceStore` on `ChainOwner`
- Owner head / safe-target / reorg follow the store after tick and import
- Gossip and sync import any parent known to the store (FC-driven tip)
- Durable `--data-dir` resume rebuilds the store from genesis + blobs
  (tip-anchor fallback when ancestors were pruned)

## Backlog (priority)

| Pri | Area | Action |
| --- | --- | --- |
| P0 | Live FC store in node | **Done** |
| P0 | Attest to safe-target | **Done** |
| P1 | FC-driven head | **Done** — known-parent import + store.head → owner |
| P1 | Resume FC | **Done** — durable replay / tip-anchor |
| P1 | Fixture re-fill | `at_9` / `dead_9` empty bodies vs BlockSpec (upstream fill) |
| P2 | Operator A2/A3 / Hive | External pins only; no invented digests |
| P2 | leanBench alignment | Comment/API mode notes when live |

## This session slice

1. FC-driven head + sync import of competing known-parent tips — done.
2. Durable FC rebuild on resume — done.
3. Version bump to **0.1.50**.
