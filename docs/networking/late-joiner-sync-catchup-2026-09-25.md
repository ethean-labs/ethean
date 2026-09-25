# Late-joiner sync gate + range follow-up (2026-09-25)

## Problem

Hive sync late-joiner runs showed head advancing while `finalized` stayed at
slot 0 (`docs/hive-testing/local-hive-run-2026-09-24.md`). Root cause in-repo:

1. Every duty tick called `sync.observe(tick.slot, tick.slot)`, wiping the
   Status-derived `peer_horizon` so lag became 0 and duties ran on a stale
   local head instead of staying suppressed during catch-up.
2. Status handshake staged a **single** blocks-by-range / root batch; no
   follow-up when the peer tip remained ahead.
3. Advertised `local_status` stayed at boot genesis bytes after imports.
4. Sync horizon ignored remote `finalized` when it was ahead of remote head
   (head-behind-finalized style Status).

## What landed

| Piece | Change |
| --- | --- |
| `ethean-sync::SyncStatus` | Peer horizon is monotonic; `observe_local` for duty ticks |
| `duty_step` / `duty_loop` | Observe **owner head**, not wall-clock slot |
| `local_status::observe_remote_status` | Horizon = `max(head, finalized)` |
| `sync_catchup` | Remember peer Status tips; stage follow-up range/root after each block response |
| `duty_network` | Refresh local Status bytes after gossip/block ingest; call follow-up |
| Windows leanVM overlay | Link `psapi` for `GetProcessMemoryInfo` |

## Still open

- Same-client 3-validator finality under Type-2 / prover load (perf; partial
  proof-queue fix in lean-crypto notes).
- Hive re-run of the sync suite (needs Docker + public `:devnet5`).
- Bad-checkpoint rejection: **in-repo majority tip** landed — see
  [majority-status-tip-bad-checkpoint-2026-09-25.md](./majority-status-tip-bad-checkpoint-2026-09-25.md).

## Smoke

```bash
cargo test -p ethean-sync --lib -- status::
cargo test -p ethean-node --lib --features libp2p-quic -- \
  duty_ticks_preserve sync_catchup:: local_status::observe_uses
```
