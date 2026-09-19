# Sync-lag duty gate threshold (2026-09-19)

## What landed

PQ Interop #37 / leanSpec #689 direction: validators must skip attestation and
proposal when local head trails wall clock by more than **4** slots.

1. **`ethean-validator`**
   - `SYNC_LAG_THRESHOLD_SLOTS = 4`
   - Duty-gate tests cover lag at threshold (allowed) and above (suppressed)

2. **`ethean-node`**
   - Live `EtheanClient` builds `ChainOwner::new(SYNC_LAG_THRESHOLD_SLOTS)`
     (was 32)
   - `duty_loop` / `wall_loop` smoke tests use the same constant

## Why

A node that is many slots behind still looking "synced" deposits LMD weight on
the wrong subtree and slows convergence. Lean’s faster finality makes this worse
than Beacon.

## Still open

- Drive `DutyView.head_lag_slots` from wall clock vs head slot on every tick
  (owner already carries `max_head_lag_slots`; ensure snapshot fills lag correctly).
- BlocksByRange stream for deep catch-up (encode scaffold landed separately).
