# Early block proof: two intervals (2026-11-02)

Follows [prover-thread-share-2026-09-30.md](./prover-thread-share-2026-09-30.md)
and [docs/lean-crypto/early-block-build-idle-pump-2026-09-29.md](../lean-crypto/early-block-build-idle-pump-2026-09-29.md).

## Why one interval was not enough

The early build started in the last interval of slot N (3.2 s of a 4 s slot)
and was held until slot N+1. At leanVM `e2592df4` a Type-2 of two components
(the default cap of one attestation data plus the proposer) is about 3.3 s,
plus about 0.5 s for the proposer's Type-1. Starting at 3.2 s, the block is
ready around 7 s — interval 3–4 of N+1 — so it is already late when it
gossips. With two aggregators on one host that delay split heads and stalled
finality.

## Change

- `try_plan_next_slot_proposal` runs in the last two intervals (`EARLY_PROPOSAL_INTERVALS
  = 2`: intervals 3 and 4 when `intervals_per_slot` is 5). Interval 2 stays
  free for leanSpec aggregation. Import and gossip still wait for the slot
  boundary.
- `prover_reserved_for_block` is true while a block job is in flight, or in
  that window when this node proposes N+1. Union merges
  (`schedule_union_merges_only`) and split recovery (`seed_known_payloads`)
  skip then. An aggregator that proposes N+1 also skips the union pass at
  interval 2 so a 4–7 s merge does not still occupy the prover at interval 3.
  Local Type-1s at interval 2 still run; they are what the block includes.

On paper, a two-component proof started at 2.4 s is ready around 6.2 s
(interval 2 of N+1) instead of 7.0 s. Empty-body proofs (~2.2–2.8 s) can land
in interval 0–1 of N+1.

## Tests

- `next_slot_build_only_runs_in_the_last_two_intervals_with_a_prover`
- `next_slot_assignment_drives_the_early_window_reservation`
