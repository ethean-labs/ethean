# leanSpec FC weights from aggregated payloads (2026-09-20)

## Goal

Match leanSpec `compute_block_weights` / LMD head extraction: latest votes come
from **aggregated payload pools**, not only the `latest_known_attestations` map.

## What landed

- `head/payload_votes.rs` — newest-slot (then larger data-root) extraction over
  known / new payload participant sets; skip heads at or below finalized
- `relevant_known_votes` / `relevant_new_votes` prefer payload extraction, with
  HashMap fallback when the pool is empty
- `block_weights_from_known` anchors at `latest_finalized.slot` (leanSpec)

On-block still records participant sets from `aggregation_bits` so block-only
fixtures keep non-zero weights (stricter leanSpec “empty proof until gossip”
deferral is not enabled on the structural path).

## Still open

- `test_finalized_safety/test_fork_above_finalized_wins_at_or_below_loses`:
  cached fixture `at_9` has an **empty** attestation body while
  `storeSnapshot.knownAggregatedPayloads` expects a 7-validator payload. Without
  that payload, LMD cannot move weight off `block_4` (want 0 / got 6). Needs a
  fixture refresh from current leanSpec fill, or a confirmed wire format for
  deferred proofs.
- Payload-pool snapshot assertions still deferred for prune exactness
