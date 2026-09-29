# Majority Status tip for bad-checkpoint rejection (2026-09-25)

## Problem

Hive `sync: head behind finalized rejects ahead bad checkpoint` connects the
client to two agreeing LeanSpec helpers plus one isolated adversarial peer
whose finalized tip is artificially ahead. Catch-up that always chased the
highest Status tip would import and finalize the adversary's checkpoint.

## What landed

| Piece | Change |
| --- | --- |
| `sync_catchup::preferred_finalized` | leanSpec `get_network_finalized_slot`: most-reported finalized slot, ties go to the higher slot; within that slot the most-reported root (see 2026-09-29 update) |
| `select_catchup_target` | Stage range/root only toward a majority tip still ahead |
| `apply_preferred_horizon` | Duty lag uses majority tip; may lower after a bad tip drops |
| `SyncStatus::replace_peer_horizon` | Non-monotonic replace for preferred-tip recompute |
| Status handshake | Remembers tip only; block fetch deferred to majority catch-up |
| `--checkpoint-sync-url` | Fail closed on fetch/verify error (leanSpec abort, no genesis fallback) |

Also retained from the same readiness pass: head-behind-finalized root pin,
`tools/hive/smoke-local.ps1`, proof-queue priority (0.1.69), genesis pins.

## Smoke

```bash
cargo test -p ethean-sync --lib -- status::
cargo test -p ethean-node --lib --features libp2p-quic -- sync_catchup::
```

## Update 2026-09-29: tie rule and disconnects

The first cut required two agreeing votes, so a 1–1 split (or a single peer)
left no preferred tip. It now follows leanSpec `PeerManager.get_network_finalized_slot`:

- count finalized slots across remembered peers and rank by `(count, slot)`,
  so an even split resolves to the higher slot;
- inside the winning slot, pick the most-reported root (larger root on a tie).

With 2 honest helpers and 1 ahead adversary the honest slot still wins 2–1.

Peers also stop voting when they leave. `PumpEvent::ConnectionClosed` carries
`last` (libp2p `num_established == 0`); the pump collects those peers,
`forget_disconnected_peers` drops them from Status sessions and sync targets,
and the duty horizon is recomputed. A peer with a second live connection keeps
its vote.

Tests: `equal_count_split_prefers_higher_slot`,
`same_slot_fork_picks_most_reported_root`, `forgotten_peer_stops_voting`,
`empty_targets_have_no_preference`.

## Still open

- Live Hive re-run (Docker Desktop + public or local `:devnet5`)
- ethereum/hive PR + A2/A3 paste (external)
