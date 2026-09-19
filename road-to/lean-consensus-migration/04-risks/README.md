# Migration Risk Program

This directory defines the release-blocking risk controls for moving Ethean from its current experimental Beam-era implementation to an interoperable Lean Consensus client. It is a planning contract, not a statement that the controls already exist.

## Current baseline

The repository currently contains important prototype-only behavior:

- `src/crypto/hash.rs` labels a SHA-256-based transformation as Poseidon and explicitly says it is simplified.
- `src/crypto/wots/sign.rs` signs directly with supplied WOTS private-key material but has no persistent XMSS leaf allocator, reservation journal, or rollback defense.
- `src/storage/database.rs` exposes batch writes, while the RocksDB implementation is a no-op placeholder and `Database::close` does not flush.
- `src/storage/checkpoints.rs` writes metadata, state, and the latest pointer separately, calculates a zero state root, and does not authenticate checkpoint provenance.
- `src/storage/migrations.rs` records migration state and schema version in separate writes and permits an interrupted partial transition.
- `src/network/gossip.rs` estimates rather than decodes message size, uses timestamp-derived message IDs, and holds in-memory queues and caches without protocol-derived byte budgets.
- Existing performance suites sleep or synthesize memory and throughput values. They cannot establish production budgets.
- `Cargo.toml` uses broad semver requirements and has no Prometheus exporter, SSZ/Snappy stack, Lean signature implementation, prover runtime, or reproducible-build policy.

These facts make crypto substitution, key rollback, untrusted decoding, persistence, and timing release blockers.

## Evidence policy

Protocol behavior is governed by the current pinned leanSpec, leanSig, leanMultisig, and selected pq-devnet artifacts. Ream, Zeam, qlean-mini, ethlambda, Lantern, gean, and Peam are independent interoperability evidence, never normative sources or code-layout templates.

Phase 00 must record immutable commit IDs, dependency lockfiles, feature flags, hardware profiles, test vectors, and observed wire behavior for every source used. The local `bazalinacaklar/` research directory contained no Markdown evidence when this plan was written, and external GitHub access was unavailable in this environment. Therefore no unverified peer implementation detail is asserted here. The first gate is to create that evidence ledger before design approval.

## Document map

- [RISK_REGISTER.md](RISK_REGISTER.md) owns risk identity, severity, evidence, mitigation, and closure.
- [SECURITY_GATES.md](SECURITY_GATES.md) defines fail-closed promotion gates.
- [PERFORMANCE_BUDGETS.md](PERFORMANCE_BUDGETS.md) turns Phase 00 measurements into measurable timing and resource limits.
- [DATA_MIGRATION_POLICY.md](DATA_MIGRATION_POLICY.md) defines atomic schema, checkpoint, and XMSS-state transitions.
- [../06-observability/README.md](../06-observability/README.md) defines the telemetry required to prove these controls continuously.

## Decision rules

1. A protocol, cryptographic, or persistent-key-state assumption without an immutable source pin is unresolved.
2. Fake or compatibility crypto may run only in tests and explicitly non-networked developer modes; a production build containing a selectable fake path fails.
3. No percentile, memory cap, queue cap, or SLO target is invented. Phase 00 derives it from repeatable measurements and records the result as a versioned budget artifact.
4. The protocol's four-second slot is the outer deadline. Internal budgets must leave measured safety margin for scheduling jitter, propagation, persistence, and cancellation.
5. Unknown, stale, missing, or ambiguous evidence fails closed for signing, checkpoint import, migration, and release promotion.
6. Every closed risk has an automated test, observable signal, owner role, review date, and rollback or containment procedure.

## Review cadence

The risk register is reviewed on every protocol/dependency pin change, storage schema change, signing-path change, prover change, and before each devnet promotion. Security gates run in CI and in a multi-node staging environment. Performance budgets are re-baselined only through an explicit reviewed change; regressions do not silently rewrite the baseline.
