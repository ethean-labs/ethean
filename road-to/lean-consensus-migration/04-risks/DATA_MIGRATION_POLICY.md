# Data Migration Policy

## Scope and invariants

This policy covers consensus state, blocks, indices, fork-choice/finality metadata, checkpoints, slashing protection, XMSS key state, schema metadata, and observability state needed for safe recovery.

At every crash boundary:

- readers observe one complete schema generation;
- canonical head, state, block, and indices refer to the same committed transition;
- a checkpoint is either absent or fully verified and activated;
- an XMSS leaf is never made available after reservation;
- migration history is append-only and truthful;
- restart can distinguish pending, committed, and abandoned work.

Current separate writes in checkpoint storage and migration management do not satisfy these invariants. The placeholder RocksDB backend is not migration-capable.

## Data classes

1. **Reconstructible:** caches, peer scores, transient queues, derived indices. These may be dropped after integrity checks.
2. **Consensus-critical:** blocks, states, finalized/head roots, fork-choice data, slashing protection. These require checksums, atomic activation, and validated recovery.
3. **Irreversible signing state:** XMSS key identity, generation, next/reserved/burned leaves, signature audit. This must never roll back with ordinary node data.
4. **Trust configuration:** network/fork identity, checkpoint trust roots, protocol and crypto pins. Changes require explicit operator action and audit.

## Migration package

Every migration ships with:

- monotonically increasing schema generation and unique migration ID;
- source and target schema hashes;
- compatible Ethean and protocol pin ranges;
- deterministic preflight, transform, validation, and activation logic;
- disk-space and runtime measurement from Phase 00;
- key-count/value-size checks and checksums;
- interruption map describing recovery at each step;
- rollback classification: reversible before activation, forward-repair after activation, or deliberately irreversible;
- test corpus covering oldest supported source, current source, corruption, and maximum-valid data.

Arbitrary executable scripts downloaded at runtime are prohibited.

## Execution protocol

1. Acquire an exclusive migration lease and prove no signer or second node instance can write.
2. Verify network identity, source generation, trust configuration, checksums, free space, backup integrity, and key-state generation.
3. Create a read-only source snapshot and content-addressed migration manifest.
4. Write target-generation records under a separate namespace. Do not mutate active records in place.
5. Persist migration intent and progress using idempotent chunks.
6. Validate counts, roots, references, canonical serialization, and sampled/full checks according to data class.
7. Atomically switch one activation pointer in the same durable transaction as committed migration metadata.
8. Fsync required database and directory metadata, reopen, and repeat validation.
9. Retain the source generation read-only for the approved rollback window, except irreversible XMSS state.
10. Resume networking first in non-signing mode; enable signing only after key-state reconciliation and readiness gates pass.

## XMSS separation and rollback

XMSS state is stored in a dedicated transactional domain with stricter durability than reconstructible consensus data. A consensus database restore never restores the leaf allocator.

Each reservation contains key ID, signing generation, leaf index, duty domain, message digest, monotonic sequence, wall-clock audit timestamp, process instance, and status. Reservation commit precedes signature construction. Cancellation, panic, timeout, or unknown completion marks the leaf burned.

Backup/restore behavior:

- backups declare their highest signing generation and are not independently sufficient to re-enable signing;
- restoring an older generation triggers permanent signing lock;
- cloning a signer store causes instance-fencing conflict; only one explicitly promoted instance may sign;
- reconciliation checks durable audit plus known network publications where available;
- uncertainty requires key rotation, never counter decrement or leaf reuse.

## Checkpoint import

A checkpoint bundle contains canonical state bytes, computed state root, block/finalized roots, slot/epoch, network and fork identifiers, protocol pin, source identity, trust-chain material, creation time, and content hashes.

Import is staged and read-only until all fields validate against the pinned specification and configured trust root. Cross-network, stale beyond policy, future, conflicting finalized history, zero/unknown state root, malformed SSZ, or unapproved source fails closed. Download transport security is not checkpoint authenticity.

Changing the trust root is a separate audited operation with operator confirmation. Checkpoint import cannot silently modify it.

## Failure and recovery

On startup, Ethean reads the activation pointer and migration intent:

- no intent: open the active generation;
- incomplete pre-activation intent: verify source and resume or discard target idempotently;
- committed activation: open target and validate; never silently fall back to source;
- ambiguous durability or checksum mismatch: remain not ready and require repair tooling;
- signer-generation mismatch: remain non-signing regardless of consensus database health.

Disk full, I/O error, permission loss, process kill, power-loss simulation, partial/corrupt values, and concurrent-start tests run at every persistence boundary.

## Retention and deletion

Deletion is a separate reviewed operation after the rollback window, successful restart, finalized progress, backup verification, and observability confirmation. Signing audit and burned-leaf records remain for the key lifetime plus the security retention period. Secure key deletion follows the key-management policy and never makes indices reusable.

## Compatibility

Only explicitly listed schema generations are accepted. Newer unknown schemas fail without mutation. Downgrades run read-only unless a tested reverse migration exists. Mixed-version rolling upgrades are allowed only if both versions share a documented read/write compatibility window and cannot violate signing-state exclusivity.
