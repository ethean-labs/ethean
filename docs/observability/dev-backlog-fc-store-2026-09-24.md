# Development backlog and next-step decision (2026-09-24)

## Current snapshot (v0.1.49)

- leanSpec FC fixture runner green (empty-body dump gates where fill skews)
- leanMetrics schema v3 + Grafana interop dashboard
- leanMultisig / leanVM wire paths present; fail-closed crypto
- Node `safe_target` / `reorg_total` exist but safe-target still tracks
  **justified** (parent-heuristic reorg only) — no live `ForkChoiceStore`

## Is embedding ForkChoiceStore necessary?

**Yes.** Lean attestations should target the interval-3 safe-target (deepest
2/3-backed block), not the justified checkpoint alone. Without a store:

- `ethean_safe_target_slot` / `lean_safe_target_slot` stay conservative stubs
- Local attesters sign head/justified, not FC safe-target
- LMD reorg counting cannot match leanSpec `reorg_total`

Full LMD head replacement of linear gossip import remains a later phase
(node still only extends `parent == head`).

## Backlog (priority)

| Pri | Area | Action |
| --- | --- | --- |
| P0 | Live FC store in node | Optional `ForkChoiceStore` on `ChainOwner`; init at genesis; `on_block` on apply; `on_tick` on duty ticks; sync safe_target/reorg |
| P0 | Attest to safe-target | `duty_attest` uses `owner.safe_target` when set |
| P1 | Fixture re-fill | `at_9` / `dead_9` empty bodies vs BlockSpec (upstream fill) |
| P1 | FC-driven head | Allow competing-tip import + store.head → owner.head |
| P2 | Operator A2/A3 / Hive | External pins only; no invented digests |
| P2 | leanBench alignment | Comment/API mode notes when live |

## This session slice

1. Tracked plan (this note) — done.
2. Wire optional structural `ForkChoiceStore` + tick/import sync — done
   (`crates/node/src/chain_fc.rs` + apply/tick call sites).
3. Point local attestation head/target at safe-target — done (`duty_attest`).
4. Version bump to **0.1.49** (this release). Node unit tests need Linux/WSL
   when leanVM `system-info` lacks Windows `rusage`.

## Remaining backlog

| Pri | Area | Action |
| --- | --- | --- |
| P1 | Fixture re-fill | `at_9` / `dead_9` empty bodies vs BlockSpec (upstream fill) |
| P1 | FC-driven head | Allow competing-tip import + store.head → owner.head |
| P1 | Resume FC | Rebuild store from durable tip (not only genesis slot 0) |
| P2 | Operator A2/A3 / Hive | External pins only; no invented digests |
| P2 | leanBench alignment | Comment/API mode notes when live |
