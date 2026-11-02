# Hot-path copies (2026-11-02)

Item 2 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

- Gossip/sync import owns the decoded `SignedBlock`. After STF and Split seed,
  the inner `Block` is moved into `fc_on_block` instead of cloned.
- Accepted gossip blocks move the wire payload into the durable queue (no
  extra `payload.clone()`).
- Serve cache indexes a verified blob once (`put_at_slot`); the by-root map
  is the same buffer.
- Type-1 insert borrows participant bits and proof bytes; the pool still
  stores one copy for later merges.
- Split recovery builds the job list first and moves the component keys into
  the last job.

`try_init_fork_choice` borrows `head_state`; `genesis_anchor` still clones
once for the unsealed header (start-up). Own-block gossip stays in
`pending_block_gossip` until publish, then one copy is queued durable and
the rest moves into the serve cache. `apply_planned_locally` moves the
block into fork choice.
