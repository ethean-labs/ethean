# Deferred vote retry and indexed block pruning (2026-09-30)

Follows [soak-memory-bounds-2026-09-30.md](./soak-memory-bounds-2026-09-30.md).

## Deferred votes

A 20-minute soak dropped 9–48 votes per node (about 0.3%) with
`unknown head block` or `attestation too far in the future`. The block they
point at had not been imported yet, or the vote arrived before the local tick
reached its slot. The store rejected them and nothing retried.

`ChainOwner` now keeps those votes in `deferred_votes` (single votes and
aggregates, at most `MAX_DEFERRED_VOTES` = 1024, oldest dropped first). Only
the four errors that clear on their own are queued: unknown head, target or
source block, and a vote ahead of the store clock. Anything else is still
dropped.

The queue is replayed after every block import and every tick. A vote leaves
once it is accepted, once its slot is finalized, or once the store clock is
more than `DEFERRED_VOTE_SLOTS` (4) past it.

## Pruning without decoding

`prune_below_floor` decoded every stored block, from the files and from redb,
to learn its slot. The slot is the first field of the block, and a signed
block starts with two 4-byte offsets followed by the block, so the slot sits
in the first 16 bytes. `slot_from_block_ssz` now reads those bytes and no
longer decodes.

redb keeps the same information in a new `ethean_block_slots` table (root →
slot, 8 little-endian bytes), written next to the block on every save. The
prune pass:

1. reads the first 16 bytes of each `blocks/*.ssz` and collects root → slot,
2. deletes the files below the floor,
3. fills any missing slot index rows from that list (rows written before the
   index existed), then deletes the redb rows below the floor by the index.

After one pass the index is complete, so later passes need the files only for
the file deletion itself.

## Tests

- `early_vote_is_applied_once_the_store_reaches_its_slot`,
  `stale_deferred_votes_are_dropped`, `queue_drops_the_oldest_when_full`,
  `only_block_and_clock_errors_are_retried`
- `slot_from_signed_block` now also checks the bare block and a full decode
- `pruning_redb_uses_the_slot_index_and_keeps_it_current`: rows saved without a
  slot index are pruned from the file headers, the kept row gains an index
  entry, and the next pass prunes it with the files already gone
