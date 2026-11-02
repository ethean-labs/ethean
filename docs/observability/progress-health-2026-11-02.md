# Progress health (2026-11-02)

Item 6 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

`/healthz` is still process-alive. `/readyz` now also fails when:

- the canonical head has not moved for 8 wall-clock slots
- a prover worker was started and the thread has exited
- a Type-2 job has been in flight for 8 slots

Gossip idle (peers connected, no inbound messages for 8 slots) is
`ethean_gossip_idle` plus a warning. It does not fail `/readyz`: a quiet mesh
is not a crash.

Gauges: `ethean_ready_head_progress`, `ethean_ready_prover_alive`,
`ethean_ready_block_proof`, `ethean_gossip_idle`.
