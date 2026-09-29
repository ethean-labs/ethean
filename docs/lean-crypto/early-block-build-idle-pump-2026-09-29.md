# Early block build and between-interval pump (2026-09-29)

## Problem

At leanVM `e2592df4` the block's Type-2 merge takes about 3.3 s on a dev
machine for two components (Type-1 about 0.5 s, cold start about 4 s, verify
about 33 ms; scaling in
[prover-timing-block-data-cap-2026-09-29.md](prover-timing-block-data-cap-2026-09-29.md)). lstar
slots are 4 s with five 0.8 s intervals. Two things made the block even later:

1. The proposer started the merge at interval 0 of its own slot, so the block
   reached peers well after the interval-1 attestation vote.
2. The mesh duty loop pumped the swarm for about 20 ms per interval and then
   slept until the next interval. A proof that finished mid-interval waited up
   to 0.8 s before collection and gossip, and inbound gossip sat in libp2p the
   same way.

`LOG_INV_RATE` stays 2. ethlambda, Ream, Grandine lean and leanSpec prod all use 2,
so lowering it is not an option.

## What changed

| Piece | Change |
| --- | --- |
| `duty_propose::try_plan_next_slot_proposal` | During the last interval of slot N, plan, sign and submit the block for N+1 when we propose it (mesh with a prover only; local-finality smoke is unchanged) |
| `ChainOwner::block_proof_slot` | Set when a block job is submitted. `try_plan_proposal` skips a slot that is already proving, so the slot is never signed twice. Cleared on any block proof failure so intervals 0..=2 can retry |
| `proof_collect::DeferredBlockProof` | A block proof that returns before its slot is held, not imported |
| `proof_collect::release_deferred_block` | Runs right after the wall tick is accepted, before attest/propose duties, and imports and queues the held block once the wall slot reaches it |
| `duty_mesh::pump_until_next_interval` | Replaces the interval sleep: keeps pumping the swarm in slices of up to 50 ms until the next interval, collects finished proofs, and flushes block and aggregation gossip straight away |
| `swarm_pump::pump_swarm_window` | Pump variant with a hard deadline so a steady trickle of events cannot push the loop past the interval boundary |

The same interval-4 pre-build is used by ethlambda (`propose_block(next_slot)` at
end of slot) and Grandine lean (`devnet-5-leanvm-main`, "pre-building block one
interval early").

## Expected effect

The block proof now starts about 0.8 s earlier and is published within one pump
slice (at most 50 ms) of finishing, instead of at the next interval boundary.
With the timings above, a proposer publishes around 2.5–2.8 s into its slot
instead of about 4 s, i.e. inside the slot rather than into the next one. It
still lands after the interval-1 vote, so attesters in that slot vote for the
parent; that holds for every client at this leanVM pin.

## Tests

- `a_slot_with_a_block_proof_in_flight_is_not_replanned`
- `next_slot_build_only_runs_in_the_last_interval_with_a_prover`
- `early_block_proof_is_held_until_its_slot` (held before the boundary, then
  verified at the boundary; a bogus proof is rejected and frees the slot)

```bash
cargo test -p ethean-node --features libp2p-quic
```

## Still open

- Live timing on a mesh (needs Docker for Hive, or a local multi-node run).
- Optional: cap aggregation jobs while our own block proof is in flight
  (ethlambda runs one job when it proposes the next slot).
