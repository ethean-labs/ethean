# Keeping the prover free for our own block (2026-09-29)

## Problem

The proof service runs one leanVM job at a time. Its inbox already ordered
jobs as Block > Attestation > Split, but three gaps remained:

1. **Order inside one duty step.** `apply_wall_step` called
   `schedule_aggregations` before the two proposal planners. The worker thread
   picked up the first Type-1 right away, so the early block job (last interval
   of slot N, for N+1) queued behind a job that was already running.
2. **Queue depth counted the block.** `QUEUE_DEPTH = 16` covered all jobs. After
   a block with many votes, split recovery plus fresh aggregates could fill the
   queue, and `request_block_proof` then failed with "prover queue is full".
3. **The interval before the early build.** A Type-1 started one interval
   before the early build could still hold the prover when the block job
   arrived. A Type-1 takes about 0.5 s at leanVM `e2592df4` (1.1–1.7 s on a
   loaded host; see
   [prover-timing-block-data-cap-2026-09-29.md](prover-timing-block-data-cap-2026-09-29.md)),
   while an lstar interval is 800 ms.

ethlambda runs a single aggregation job when the node proposes the next slot,
and gean lowers its aggregation cap for a proposing aggregator. Both address the
same contention.

## What changed

| Piece | Change |
| --- | --- |
| `duty_step::apply_wall_step` | Plans proposals (current slot, then the early next-slot build) **before** scheduling aggregations |
| `proof_service::PriorityInbox::has_room_for` | Depth only counts attestation and split jobs. A block job is accepted whenever no other block is queued |
| `duty_propose::holds_prover_for_next_block` | True in interval `intervals_per_slot - 2` when this node has a prover and a local proposer, is not in local-finality mode, owns slot N+1, has no block for it yet, and is not already proving it |
| `duty_step` | While that holds, an aggregator skips new aggregation for one interval (`lean_aggregator_skipped_total{reason="other"}`, plus a debug log). The signatures stay pooled; at the last interval they are scheduled after the block job and queue behind it |

Proofs that are already running are not preempted; the prover protocol has no
cancel. Aggregates delayed this way publish after our block is proved. They
remain valid pool entries for later blocks.

Later the same day aggregation moved to interval 2 only (leanSpec
`tick_interval`), so nothing new starts in interval 3 and
`holds_prover_for_next_block` was removed; see
[safe-target-merge-block-data-cap-1-2026-09-29.md](../lean-spec/safe-target-merge-block-data-cap-1-2026-09-29.md).

## Tests

- `a_full_aggregation_queue_never_refuses_the_block`: 16 queued attestation
  jobs still admit a block job, only one block may wait, and it pops first.
- `shared_inbox_respects_depth`: attestation and split jobs are still refused
  beyond the depth.
- `the_interval_before_an_early_build_keeps_the_prover_free`: the gate holds
  only in interval 3 of 5, with a prover, when we own the next slot, and it
  does not hold while that slot is already being proved or in local finality.

```bash
cargo test -p ethean-node --features libp2p-quic --lib proof_service
cargo test -p ethean-node --features libp2p-quic --lib duty_propose
```
