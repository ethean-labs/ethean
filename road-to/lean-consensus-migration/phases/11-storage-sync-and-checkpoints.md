# Phase 11 — Storage, Sync, and Checkpoints

## Pinned inputs

- Snapshot `LC-D5-2026-09-19`; Ethean baseline `880982f`; `leanSpec` `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54`; Phase 00 fixture SHA-256 manifest.
- Consensus objects and roots are the Phase 03 canonical SSZ schema; fork-choice/finality behavior is the Phase 06 profile.
- Network retrieval uses the Phase 10 protocol IDs, maximum 1,024 blocks per request, and profile-derived network/genesis fingerprint.
- Crypto pins are `leanSig` `15cbdd43ec8525aa43fea2f42cafc5ed366084ae` and `leanVM` `e2592df4e30fdddbbf8ae26a333116c68cec7026`.
- Rust `1.97.1`, RocksDB crate/native version, lockfile, schema ID `ethean-lc-d5-v1`, migration binary digest, and runtime image digest are frozen for the phase.
- Peer evidence: Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, gean `b78f6d737f4df57a72d5e230635681235fda8024`, and Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.

## Objective

Persist validated Lean data atomically, reconstruct deterministic chain state after restart, synchronize missing ancestry safely through parent and range sync, prune without losing recovery data, and make checkpoint trust an explicit operator boundary.

## Non-goals

- No legacy JSON/database migration, archive-node guarantee, Beacon weak-subjectivity formula, execution-layer coupling, or claim that an internally consistent checkpoint is canonical.
- No serving unverified pending data and no automatic fallback from a rejected checkpoint to unsafe state.
- No backend abstraction that hides durability semantics required by signer or chain commits.

## Entry criteria

- Phases 03–10 pass; canonical SSZ, transactional transition, finality, signer journal, duties, and retrieval interfaces are stable.
- Storage invariants identify each atomic boundary and required fsync behavior.
- Retention budgets cover finalized history, side branches, states, votes, proofs, pending objects, signer state, and API visibility.
- Checkpoint source authentication and operator trust policy are approved.

## Exact old and new paths

Replace then delete:

- `src/storage/` including no-op `RocksDbBackend`, JSON records, zero-root checkpoints, caches, indices, backup, benchmark, and migrations.
- `src/integration/sync_coordinator.rs`, `src/integration/network_storage.rs`, and storage wiring in `src/client.rs`.

Create:

- `crates/ethean-storage/src/{lib.rs,db.rs,batch.rs,schema.rs,errors.rs,recovery.rs,prune.rs}`.
- `crates/ethean-storage/src/tables/{blocks.rs,states.rs,indices.rs,metadata.rs,votes.rs,pools.rs}`.
- `crates/ethean-sync/src/{lib.rs,status.rs,parent.rs,range.rs,backfill.rs,checkpoint.rs,trust.rs}`.
- `tests/recovery/storage_crash.rs`, `tests/recovery/pruning.rs`, `tests/interop/sync.rs`, `tests/negative/checkpoint.rs`, and `tools/db-inspect/`.

New code directories require English `README.md` files; all hand-written source files are at most 300 lines.

## Ordered tasks

1. Define typed tables for immutable block components, post-states/diffs, parent/child and slot/root indices, canonical metadata, checkpoints, latest votes, bounded pools, signer watermarks, profile, and schema version.
2. Store only canonical SSZ or explicitly versioned local records with checksums; never serialize consensus identity through JSON.
3. Commit block, post-state, indices, and import marker in one durable batch before publishing an accepted event. Canonical head/checkpoints update only after referenced data is durable.
4. Keep signer transactions independent but order shutdown/recovery so chain rollback cannot rewind signer watermarks.
5. On startup verify schema/profile/network, metadata references, roots, parent links, state reconstruction, checkpoint monotonicity, and signer watermark; quarantine corruption and fail closed.
6. Rebuild fork choice from the finalized anchor plus retained descendants and latest votes, then compare reconstructed head/checkpoints with the persisted deterministic snapshot.
7. Implement parent sync by root with bounded depth, deduplication, retry/backoff, peer rotation, and validation before persistence.
8. Implement forward range sync with ordered batches, skipped-slot handling, partial-response retries, competing-fork detection, importer backpressure, and a hysteretic sync/duty gate.
9. Implement backfill below the anchor without changing head, justified/finalized checkpoints, signer state, or live duties.
10. Accept checkpoint bundles containing network/profile identity, finalized block, post-state, roots, source metadata, timestamp, and optional authenticated signatures. Verify structure and state/block pairing separately from trust.
11. Require an operator-pinned checkpoint root or quorum policy. Label URL transport, TLS identity, source quorum, and canonical-membership assumption in logs/API/evidence.
12. Install checkpoints through a staged directory, fsync files/directories, atomically switch the active generation, retain the prior generation until successful restart, then backfill.
13. Prune only below the finalized policy floor while retaining reconstruction dependencies, pending-parent ancestry, proof verification data, signer journals, checkpoint generations, and configured API history.
14. Exercise disk full, read-only, bit rot, partial batch, stale backup, schema mismatch, and kill points with deterministic recovery.

## Deletion obligations

- Delete the production-named no-op database, JSON typed storage, zero state-root checkpoint, unsafe backup assumptions, and generic sync jobs.
- Refuse legacy Panro/Beacon data directories with a clear error and clean-genesis/checkpoint instructions.
- No dual-write, lazy legacy conversion, or production in-memory fallback remains.
- Repository scans may mention deleted paths only in migration/deletion records.

## Security/spec risks

- Acknowledged but non-durable writes can violate finality and signer safety.
- Checkpoint consistency does not prove canonical-chain membership; source compromise is a trust failure, not a decode success.
- Pending blocks can turn a node into a distributor of unverified attacker data.
- Pruning can remove ancestry required for proof verification, reorg handling, restart, or backfill.
- Range responses can be partial, duplicate, out of order, or forked.
- Corrupt metadata may point at missing data; recovery must never invent roots or regress checkpoints.

## Positive and negative fixtures

- Positive: canonical SSZ round trips, block/state/root pairing, skipped slots, finality/pruning boundaries, profile/schema fingerprints, and trusted checkpoint bundles with explicit source metadata.
- Negative: foreign network, legacy JSON DB, unknown schema, missing state, root mismatch, orphan index, nonmonotonic checkpoint, untrusted source, malformed bundle, partial range, and serve-before-validate.
- Fixture provenance and hashes live in `tests/fixtures/storage/manifest.toml` and the Phase 00 SHA-256 manifest.

## Interop and differential tests

- Genesis sync, parent recovery, range catch-up, checkpoint bootstrap, backfill, reorg, restart, and finalization against each available pinned peer.
- Scale: long finality gap, maximum retained forks/pools, large skipped-slot range, and bounded startup reconstruction.
- Recovery: kill before/after each batch/fsync/rename, checkpoint install, pruning batch, fork-choice snapshot, signer watermark read, and range import; inject bit rot and disk exhaustion.

## Validation commands

```text
cargo +1.97.1 test -p ethean-storage --locked
cargo +1.97.1 test -p ethean-sync --locked
cargo +1.97.1 test --test storage_crash --release --locked
cargo +1.97.1 test --test pruning --release --locked
cargo +1.97.1 test --test checkpoint --release --locked
cargo +1.97.1 test --test sync --release --locked --features interop
cargo +1.97.1 clippy -p ethean-storage -p ethean-sync --all-targets --locked -- -D warnings
rg -n "RocksDbBackend|serde_json.*(Block|State|Checkpoint)|zero.*root" crates src
```

## Exit criteria

- Fault injection at every durability boundary reconstructs the same valid chain or fails closed with an actionable corruption report.
- Checkpoint API/UI always displays trust source and never claims canonicality from structural checks alone.
- Sync converges after partial/malicious peers without checkpoint regression or unbounded pending data.
- Pruning preserves restart, proof verification, configured history, and backfill invariants.
- No-op RocksDB, JSON typed storage, and legacy sync paths are deleted; touched source files are within 300 lines.

## Rollback/data policy

There is no in-place conversion from the legacy database. Schema upgrades use copy/stage/verify/atomic-switch and retain one prior generation. Code rollback may open data read-only only when it explicitly supports the schema/profile; otherwise use the retained generation or resync from a trusted checkpoint. Signer journals are outside chain rollback and are never restored backward.

## Artifacts/evidence

- Schema manifest, table inventory, durability model, fault-injection matrix, recovery hashes, corruption corpus, pruning proof, checkpoint trust statement, sync traces, disk/IO profiles, and legacy-rejection output in `artifacts/phase-11/`.

## Dependencies

Depends on Phases 03–10. Phase 12 exposes bounded operational surfaces; Phase 13 qualifies upgrade, rollback, soak, and release artifacts.
