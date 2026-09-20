# Phase 11 — Storage, sync, and checkpoints (2026-09-19)

## Summary

Phase 11 adds durable storage contracts and sync/checkpoint planners.

### ethean-storage
- Schema `ethean-lc-d5-v1`
- `WriteBatch` + flush marker; SSZ bytes + SHA-256 checksums
- Non-rewinding signer watermark; legacy Panro/JSON paths refused
- `PrunePolicy` + `recover_on_open` orphan metadata quarantine
- RocksDB native backend deferred (process-local `Database`)

### ethean-sync
- `SyncStatus` hysteresis (high 8 / low 2) for duty gating
- Parent sync planner (depth + dedupe)
- Range planner (≤1024)
- Backfill invariant (no head/finalized/signer mutation)
- `CheckpointBundle` structure vs `TrustPolicy` (pinned root / quorum)

## Tests
- storage **10**, sync **7**

## Open
RocksDB fsync backend, crash-inject matrix, `tools/db-inspect`, live peer import loop.
