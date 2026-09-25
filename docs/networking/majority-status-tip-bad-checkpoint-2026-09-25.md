# Majority Status tip for bad-checkpoint rejection (2026-09-25)

## Problem

Hive `sync: head behind finalized rejects ahead bad checkpoint` connects the
client to two agreeing LeanSpec helpers plus one isolated adversarial peer
whose finalized tip is artificially ahead. Catch-up that always chased the
highest Status tip would import and finalize the adversary's checkpoint.

## What landed

| Piece | Change |
| --- | --- |
| `sync_catchup::preferred_finalized` | Prefer any finalized `(root, slot)` with **≥2** peer votes |
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

## Still open

- Live Hive re-run (Docker Desktop + public or local `:devnet5`)
- ethereum/hive PR + A2/A3 paste (external)
