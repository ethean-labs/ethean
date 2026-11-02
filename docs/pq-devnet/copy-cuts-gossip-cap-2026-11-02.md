# Fewer copies on the proposal and gossip paths (2026-11-02)

Follows [early-block-two-intervals-2026-11-02.md](./early-block-two-intervals-2026-11-02.md).

## Copies that were not needed

- `try_plan_proposal` cloned the whole head `State` (registry, justifications,
  historical hashes) even though `plan_from_pool` only needs a borrow. It now
  clones the small `ChainProfile` and reads the state in place.
- `request_block_proof` cloned each Type-1 proof (~180 KB) into the prover job
  while the `PlanTransition` still held the same bytes. The job takes the
  proofs; `accept_block_proof` only needs the block body.
- Gossip block import cloned `pre.validators` for `seed_known_payloads` after
  `apply_block(&pre, ...)`. The parent state is still in scope, so the seed
  call borrows it.

## Gossip backlog

Votes and aggregates waiting for QuicSwarm publish lived in an unbounded
`Vec`. A stalled swarm would keep ~180 KB aggregates forever.
`queue_aggregation_gossip` keeps at most `MAX_PENDING_AGGREGATION_GOSSIP`
(32) and drops the oldest.

## Tests

- `pending_aggregation_gossip_drops_the_oldest`
- existing `duty_propose` / `gossip_stf` tests
