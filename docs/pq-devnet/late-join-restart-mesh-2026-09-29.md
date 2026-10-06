# Late joiner and restart on the local proving mesh (2026-09-29)

Two operator scenarios added to `scripts/local-devnet.ps1` and the fixes they
forced. Setup: 4 Ethean nodes, 1 validator each, PROD XMSS keys, leanVM
prover `e2592df4`, one 20-core Windows host (see
[local-proving-devnet-windows-2026-09-29.md](./local-proving-devnet-windows-2026-09-29.md)).

```powershell
.\scripts\local-devnet.ps1 -Nodes 4 -RunSeconds 330 -PollSeconds 15 -DebugLog `
  -LateJoin 3 -LateJoinAt 90 -Restart 2 -RestartAt 190 -RestartDown 20
```

- **Late join:** node 3 starts at genesis + 90 s with
  `--checkpoint-sync-url http://127.0.0.1:5052` (node 0's finalized pair).
- **Restart:** node 2 is hard-killed at + 190 s and started again 20 s later
  from the same `--data-dir`, without `--reset-chain`
  (logs `ethean_2.restart.{out,err}.log`).

## First run: both scenarios failed

| Node | Symptom | Cause |
| --- | --- | --- |
| 3 (checkpoint) | Stuck at slot 17, thousands of empty blocks-by-root replies | Peers only cached their own proposals for req/resp; gossip-imported blocks were never served |
| 2 (restart) | Came back at slot 0 | The `config.yaml` start path ignored `--data-dir`; nothing was persisted |

Justification stopped at 46: with node 2 down and node 3 off the chain, only
2 of 4 validators voted.

## Second run: joiner on a private fork

After the serve-cache and data-dir fixes node 2 resumed (fork choice rebuilt
from 42 durable blocks, range catch-up, back on the head within 15 s). Node 3
reported a head close to the others, but it was its own chain: every 4th slot
(its validator index), justified/finalized frozen at the anchor (16). None of
the other nodes' blocks imported ("parent not importable"). Because its votes
carried a stale source, the other three had to vote unanimously to justify;
finalization stayed at 16 for the rest of the run.

Two network bugs kept it from ever syncing:

1. `flush_status_outbox` / `flush_blocks_outbox` / `flush_blocks_range_outbox`
   took the whole outbox and returned on the first send error. One Status
   session for a peer that had just dropped sat in front of the two live
   peers and every flush discarded them (974 "peer not connected" lines, zero
   completed handshakes, so range catch-up never started).
2. The pump window reported connects and disconnects as two lists, applying
   disconnects first. A peer that connected and dropped inside one window was
   re-queued for Status after being forgotten.

Separately, a gossip block with an unknown parent was simply dropped; only
blocks-by-root responses buffered orphans and fetched parents.

## Fixes

- `send_each` (network `swarm.rs`): every staged request is attempted; the
  flush errs only when nothing went out. Used by all three outboxes.
- `PumpBudgetResult::note_connected` / `note_disconnected`: a peer whose last
  connection closed is removed from `connected_peers` of the same window.
  Pending Status sessions whose peer the swarm no longer knows
  (`SwarmFacade::is_peer_connected`) are dropped before each flush.
- `blocks_sync::buffer_gossip_orphans`: accepted block gossip whose parent has
  no state is buffered in `sync_orphans`, and the parent is requested by root
  from the gossiping peer. Existing orphan draining imports the chain once the
  parents arrive.
- `serve_cache_seed::imported_gossip_blocks`: blocks that gossip actually
  imported go into `put_block_at_slot` (serves both by-root and by-range).
- `chain_persist::pin_network_genesis` + `EtheanClient::attach_network_data_dir`:
  on the `config.yaml` path the data dir is bound to the network genesis
  (another genesis is refused with a `--reset-chain` hint), the durable head
  and fork-choice store are resumed, and `persist_dir` is set so blocks flush.

## Third run (all fixes)

| Time from genesis | Mesh (nodes 0–2) | Node 3 |
| --- | --- | --- |
| 90 s | h=20 j=16 f=16 | starts from checkpoint 16 |
| 105 s | h=29 j=22 f=16 | h=29 j=22 f=16 (synced) |
| 170 s | h=44 j=41 f=36 | h=43 j=36 f=16 → f=36 one poll later |
| 190–210 s | node 2 down, then restarted at h=48 | follows |
| 225 s → end | all four identical | h=82 j=72 f=36 at stop |

- Node 3: all four Status handshakes completed within 0.4 s of joining;
  3 range requests + 2 orphan parent fetches, 20 sync imports.
- Node 2 after restart: 9 sync imports, 6 orphan parent fetches, identical
  head within one poll.
- WARN lines: range serves reporting skipped (empty) slots as missing, and one
  block proof on each rejoining node dropped as stale while catching up.

Finalization lags justification after the restart: with one of four
validators offline every remaining vote is needed, and justification jumped
45 → 66 → 72, skipping the justifiable slot 48 that would have finalized 45
(3SF-mini needs consecutive justifiable checkpoints). That is expected under a
25 % outage on a 4-validator set, not a client fault.

## Tests

- `chain_persist::pin_network_genesis_accepts_same_and_rejects_other`
- `serve_cache_seed::imported_gossip_blocks_keeps_only_known_roots`
- `blocks_sync::gossip_orphan_asks_its_sender_for_the_parent`
- `swarm_pump::peer_that_drops_inside_the_window_is_not_connected`
- network `send_each_tests` (dropped peer does not block later requests; errs
  only when nothing went out)
