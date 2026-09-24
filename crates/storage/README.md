# ethean-storage

Durable persistence for Lean Consensus (Phase 11).

- Schema `ethean-lc-d5-v2` (forward-only)
- SSZ bytes + checksums (no JSON consensus identity)
- Explicit flush before publish; signer watermark never rewinds
- Prune below finalized floor with protected recovery roots
- RocksDB native backend remains a follow-up; process-local `Database` is the contract surface
