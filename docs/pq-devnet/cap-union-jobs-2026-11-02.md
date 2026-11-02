# Cap concurrent union merges (2026-11-02)

Item 4 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

On a 4-node / 2-aggregator host a union merge was as expensive as a Type-2
(~3 s). Queueing more than one Type-1-class job on the same prover starved
the early block proof (finalized 0 unless the prover had 8 threads).

`schedule_union_merges` now:

- does nothing while any attestation/split is already in flight
  (`MAX_IN_FLIGHT_UNIONS = 1`)
- submits at most one union per tick

Unions stay off the next-slot proposer path. This host did not re-run the
2-aggregator layout.
