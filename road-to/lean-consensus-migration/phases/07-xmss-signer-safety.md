# Phase 07 — XMSS Signer Safety

## Pinned inputs

- Phase snapshot: `LC-D5-2026-09-19`; Ethean planning baseline `880982f`.
- `leanSpec` `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54`; fixtures must be generated from that tree and accepted only with the Phase 00 SHA-256 manifest.
- `leanSig` `15cbdd43ec8525aa43fea2f42cafc5ed366084ae`.
- `leanVM` `e2592df4e30fdddbbf8ae26a333116c68cec7026`.
- Crypto profile: generalized XMSS, aborting target sum, Poseidon1, dimension 46, Winternitz base 8, lifetime `2^32`, 52-byte public keys, 2,536-byte signatures, and separate attestation/proposal keys.
- Protocol profile: 4-second slots, five intervals, at most 4,096 validators. Rust is `1.97.1`; `Cargo.lock` and build-image digest are immutable within the phase.
- Peer evidence is fixed to Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, and gean `b78f6d737f4df57a72d5e230635681235fda8024`.

## Objective

Deliver a role-separated signer whose one-time signing state survives concurrency, crashes, backups, and operator rollback. A signature may leave the process only after its leaf and duty reservation are durable.

## Non-goals

- No BLS compatibility, EIP-2335 compatibility claim, remote signer protocol, HSM integration, key ceremony, or key rotation beyond the pinned profile.
- No aggregate proof production; Phase 08 consumes verified raw signatures.
- No production support claim. Encrypted-at-rest deployment policy is a release gate, not a reason to weaken signer invariants.

## Entry criteria

- Phases 00–06 have passed; canonical signing roots, slot arithmetic, validator records, chain profile, and finality view are stable.
- The fixture manifest identifies production crypto separately from fake/Shadow vectors.
- Storage exposes atomic compare-and-set or serializable transactions plus durable flush.
- Threat model covers copied data directories, stale snapshots, concurrent duty triggers, clock rollback, process kill, disk full, and partial writes.

## Exact old and new paths

Replace and then delete:

- `src/crypto/wots.rs`, `src/crypto/wots/`, `src/crypto/hash.rs`, and `src/crypto/bls.rs`.
- BLS/signing flows in `src/consensus/attestation_processing.rs`, `src/consensus/finality.rs`, `src/consensus/slashing.rs`, `src/api/validator.rs`, and `src/types/validator.rs`.

Create:

- `crates/ethean-crypto/src/{lib.rs,xmss.rs,bindings.rs,errors.rs}`.
- `crates/ethean-validator/src/signer/{mod.rs,duty.rs,journal.rs,keystore.rs,recovery.rs}` and `crates/ethean-validator/src/signer/README.md`.
- `crates/ethean-storage/src/tables/{signer_state.rs,duty_reservations.rs}`.
- `tests/fixtures/xmss/`, `tests/security/signer_rollback.rs`, and `tests/recovery/signer_crash.rs`.

Every hand-written source file is at most 2000 lines; split by responsibility before review.

## Ordered tasks

1. Wrap only the pinned `leanSig` API; assert serialized sizes and parameter fingerprint at build and startup.
2. Define typed `SigningRole`, `SigningDuty`, `SigningRoot`, `KeyId`, leaf position, activation window, and reservation status. Role confusion must be unrepresentable.
3. Import attestation and proposal keys through explicit metadata, restrictive file permissions, checksum verification, public-key derivation, registry matching, and zeroizing secret buffers.
4. Bind each key record to network fingerprint, crypto revision, role, validator index, activation range, and monotonically increasing journal generation.
5. Implement one serializable transaction that rejects an existing conflicting duty, reserves the leaf, writes the signing root, advances signer state, and durably flushes before signing.
6. Generate the signature from the reserved state; persist completion and signature hash. Publication receives only completed records.
7. Make exact repeat requests idempotent by returning the prior signature; reject same role/slot with a different root and all cross-role key use.
8. Serialize signer commands through a bounded owner task and retain database-level conflict checks so a second process cannot bypass the owner.
9. On startup, reconcile journal, key state, registry, wall slot, and completed reservations. Burn uncertain leaves; never rewind counters.
10. Add signed backup manifests with monotonic generation and highest reserved leaf. Reject restore below the durable watermark unless all affected keys are permanently disabled.
11. Add lifetime forecasts and hard-stop thresholds; exhausted, not-yet-active, corrupt, mismatched, or unsupported keys cannot sign.
12. Emit bounded metrics/events for reservation outcome, latency, lifetime remaining, recovery burns, and conflict rejection without key bytes, roots, validator IDs, or signatures as labels.

## Legacy deletion obligations

- Remove `blst`, `blstrs`, `bls12_381`, local WOTS, and pseudo-Poseidon dependencies after fixture parity.
- Remove old validator key fields and all “non-empty signature” or deterministic mock-key acceptance.
- No dual production signer, legacy key decoder, automatic conversion, or fallback remains.
- Repository scans may mention the old paths only in migration/deletion records.

## Security and specification risks

- XMSS is stateful; a copied or rolled-back database can create catastrophic leaf reuse.
- The pinned branch name is not authority; the full commit and parameter fingerprint are.
- OS flush semantics, network filesystems, VM snapshots, and multi-process access can invalidate apparent atomicity.
- Slot-bound leaf selection does not replace durable double-sign protection.
- Secrets can leak through debug formatting, crash dumps, metrics, backups, or proof witnesses.
- Poseidon/XMSS assumptions remain research risks; parameter substitution fails closed.

## Tests

- Fixtures: deterministic key decode, public-key derivation, valid signatures, wrong message/slot/key/role, activation edges, final leaf, exhausted key, malformed serialization, and exact byte sizes.
- Negative: role swap, conflicting root, duplicate process, unsupported pin, permissive key file, corrupt journal, disk full, stale network fingerprint, and test backend in a release build.
- Recovery: kill before reservation flush, after reservation, during signing, before completion, during backup, and during restore; each restart either returns the same signature or burns the leaf.
- Interop: Ethean signs and each pinned peer verifies; peer fixture signatures verify in Ethean; key/public bytes and rejection boundaries match.

## Validation commands

```text
cargo +1.97.1 test -p ethean-crypto --locked
cargo +1.97.1 test -p ethean-validator signer --locked
cargo +1.97.1 test --test signer_rollback --locked
cargo +1.97.1 test --test signer_crash --locked
cargo +1.97.1 clippy -p ethean-crypto -p ethean-validator --all-targets --locked -- -D warnings
cargo deny check
rg -n "blst|bls12_381|RealBLS|Wots|POSEIDON prefix" Cargo.toml Cargo.lock crates src
```

## Exit criteria

- All production signatures pass pinned vectors and cross-client verification.
- Crash matrix proves no conflicting signature and no leaf reuse across 1,000 fault-injected runs per boundary.
- Restore rollback is rejected, exact retry is idempotent, and uncertain state only advances.
- Release features cannot select fake crypto; secret-bearing types have no `Debug` output.
- Old crypto and signer paths and dependencies are deleted; every touched source file is at most 2000 lines.

## Rollback and data policy

Code rollback cannot roll signer data backward. Schema migration is forward-only and preserves a signed pre-migration watermark. If deployment fails after any reservation, restore the new journal with the old binary only if that binary understands the schema; otherwise disable the keys and recover on the new code. Key material, journals, and backups are never auto-converted or discarded.

## Evidence artifacts

- Pin/parameter report, fixture SHA-256 manifest, cross-client verification matrix, permission audit, secret-log scan, fault-injection report, journal state-machine diagram, and key-lifetime forecast.
- Store under `artifacts/phase-07/` in CI; publish hashes and summaries, never secret material.

## Dependencies

Depends on Phases 00–06 and atomic storage primitives. Phase 08 consumes safe signatures; Phase 09 consumes signer commands; Phases 11–13 validate persistence, operations, and release policy.
