# Soak run: bounded node memory (2026-09-30)

## Problem

A 20-minute local devnet (4 nodes × 3 validators, 1 subnet, 1 aggregator)
finalized normally, but each `ethean` process grew its private memory by about
6 MB per minute (280 → 410 MB). At 4 s slots that is roughly 8.6 GB per day.

Five stores only ever grew:

| Store | Growth |
| --- | --- |
| Blocks-by-root/range serve cache (`QuicSwarm`) | Every block (~238 KB with its proof) stored twice (by root and by slot), never evicted |
| Outbound request tracker | Entries added per request, never completed |
| Fork-choice store | Every block and post-state since genesis |
| Gossip message-id set | Every accepted message id |
| Orphan buffer | Also took blocks at or below the finalized slot |

At 15 blocks per minute, the serve cache alone accounts for about
15 × 238 KB × 2 ≈ 7 MB per minute.

## Changes

- **Serve cache** (`crates/network/src/serve_cache.rs`): bytes stay in memory
  for the newest 64 slots (`SERVE_BYTES_SLOTS`). The slot → root index covers
  3600 slots (`SERVE_INDEX_SLOTS`, leanSpec `MIN_SLOTS_FOR_BLOCK_REQUESTS`).
  Older roots are read through a `BlockLoader`. With `--data-dir`, the node
  installs a loader over `blocks/<root hex>.ssz`
  (`serve_cache_seed::block_file_loader`). It reads files instead of redb, so
  the database is not opened twice in one process. Blocks known only by root
  are capped at 64.
- **Request tracker** (`crates/network/src/reqresp/tracker.rs`): each Status,
  blocks-by-root or blocks-by-range response closes the oldest open request of
  that peer. A disconnect drops the peer's requests. Anything older than
  `REQUEST_EXPIRY` (30 s) is dropped with a debug log.
- **Fork-choice history** (`ForkChoiceStore::prune_finalized_history`, in
  `crates/fork-choice/src/prune.rs`): keeps the finalized block, its
  descendants and their states. It skips when the finalized root is missing
  or not an ancestor of the justified checkpoint. The store never calls it
  itself, because the leanSpec fixtures compare the full block set. The node
  calls it from `sync_from_fork_choice` whenever finalization moves past the
  last pruned slot, which also covers finalization advanced by `on_tick`.
- **Gossip ids** (`SeenIds`, `crates/network/src/gossip/validation.rs`): two
  generations of 65536 ids. When the current one fills up it becomes the
  previous one, so a duplicate is still caught for at least 65536 messages.
- **Orphans**: gossip and sync blocks at or below `ChainOwner::finalized_slot()`
  are no longer buffered waiting for a parent.
- **Durable flush** (`client_data_dir.rs`): the flush runs every interval.
  Before this change it rewrote `state.ssz`, the head root and redb each time,
  and the prune pass decoded every stored block from both the files and redb.
  Now it returns early when the head root and finalized slot are unchanged and
  there are no new blocks. The prune runs only after the floor has advanced
  `PRUNE_FLOOR_STEP` (32) slots.

The serve window on disk is still bounded by `--prune-keep-slots` (default 256
below finalized), not by 3600 slots. Range requests older than that come back
short. Operators who want the full leanSpec window can raise the flag; at
~238 KB per block, 3600 slots are about 850 MB. Without `--data-dir` there is
no loader and only the last 64 slots are served.

## Soak results

Same host, same bundle, `local-devnet.ps1 -Nodes 4 -ValidatorsPerNode 3
-RunSeconds 1200 -PollSeconds 60 -DebugLog`. Average private memory of the four
`ethean` processes, sampled every minute:

| Build | Start | End (≈19 min) | Trend | Head / finalized at end |
| --- | --- | --- | --- | --- |
| Before | 280 MB | 410 MB | ~6 MB/min, linear | 306 / 273 |
| Tracker + fork-choice pruning only | 297 MB | 404 MB | ~5.7 MB/min, linear | 307 / 253 |
| All changes above | 281 MB | 323 MB | flat at 320–330 MB after ~12 min | 307 / 298 |

The middle run confirms that the tracker and fork-choice history were small
leaks; the serve cache was the main one. With all changes, the fork-choice
store held 6–7 blocks (`pruned=4 kept=7 finalized=298`). Each node ran one
durable prune pass (floor 38, 28 files and 28 redb rows). There were no
expired requests and no failed proof jobs.

`ethean-prover` processes move between 180 and 500 MB with proof jobs and show
no trend.

## Remaining notes

- About 9–48 votes per run (≈0.3%) are skipped by fork choice as `unknown head
  block` or `attestation too far in the future`, and are not retried. The
  aggregates still carry them; a small retry queue keyed on the missing head
  would recover them.
- `prune_below_floor` still decodes every stored block to read its slot. It
  now runs once per 32 floor slots, so the cost is small; a slot → root index
  in redb would remove it.

## Tests

- `old_blocks_leave_memory_but_stay_indexed`, `index_is_bounded_to_the_serve_window`,
  `unslotted_blocks_are_capped` (serve cache)
- `response_closes_the_oldest_request_of_that_peer`, `unanswered_requests_expire`
- `pruning_keeps_only_the_finalized_block_and_its_descendants`
- `seen_ids_keep_two_generations`
- `file_loader_reads_persisted_blocks_only`
- `prune_runs_in_floor_batches`, `flush_skips_an_unchanged_head`
