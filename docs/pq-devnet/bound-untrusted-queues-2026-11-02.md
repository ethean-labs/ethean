# Bound remaining untrusted queues (2026-11-02)

Item 7 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

Already capped: serve cache, request tracker, SeenIds, aggregation gossip 32,
sync orphans 64, deferred votes 1024, durable blocks 64, signature pool 256
data roots.

This pass:

- `known_payloads` is capped at `MAX_TRACKED_DATA` (256) and pruned every
  accepted tick with the aggregates and signature pools (`max_head_lag_slots`).
- The Type-1 pool drops the oldest key when it would exceed 256 keys.
- Fork-choice `block_states` still follows `prune_finalized_history` (spec
  fixtures compare the live tree). Vote/payload maps prune on every head
  sync, not only when finalization advances.

No soak.
