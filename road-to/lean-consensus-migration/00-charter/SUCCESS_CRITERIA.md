# Success Criteria

## Program-level definition of done

The migration is complete only when all criteria below are evidenced in CI and release artifacts for one frozen compatibility profile.

1. The shipped product identifies only as **Ethean Lean Consensus Client**.
2. No old source file has retain status or remains reachable through a compatibility fallback.
3. Every baseline path is closed as replace-then-delete, rename-by-replacement, or delete.
4. The binary, lockfile, generated code, fixtures, genesis assets, and deployment image identify the same frozen profile.
5. All mandatory conformance suites pass with zero unknown fixture types and zero unapproved skips.
6. Mixed-client tests demonstrate the selected protocol profile rather than only Ethean-to-Ethean communication.
7. Crash, rollback, malformed input, network abuse, and proof-resource tests pass.
8. Long-running four-second-slot operation stays within approved CPU, RAM, disk, file-descriptor, network, and duty-deadline budgets.
9. The old package identity, Beacon protocol assumptions, BLS, JSON roots, pseudo-Poseidon, local unpinned WOTS, mock production networking, no-op persistence, and placeholder APIs are absent.

## Phase 00 acceptance

Required artifact: an immutable compatibility ledger.

The gate passes only if it records exact values and provenance for:

- `leanSpec` commit, fixture release, and fixture SHA-256;
- devnet/network name, genesis assets, fork/topic identity, and quickstart pin;
- all consensus containers, field order, bounds, roots, and signed-object identities;
- `MAX_ATTESTATION_DATA` and duplicate-data rules;
- finality/fork-choice generation, quorum, tie-break, vote promotion, and interval behavior;
- `leanSig` and `leanVM`/`leanMultisig` revisions and complete parameters;
- proposal/attestation key encodings, signature size, lifetime index, activation, exhaustion, and rotation;
- Type-1/Type-2 statement, participant ordering, child-proof rules, component ordering, proof limit, and `log_inv_rate`;
- gossip topics, message-ID bytes, Snappy mode, decompression cap, Gossipsub profile, protocol IDs, framing, and status codes;
- checkpoint trust policy and signer-state durability model;
- production/test crypto separation;
- target hardware and proof/duty resource budgets.

Acceptance evidence includes canonical fixtures, exact source links, at least two peer comparisons where implementations exist, and an explicit resolution for each disagreement. An unspecified value fails the phase.

## Phase 01 acceptance

- Package, library, binary, CLI, logs, and version metadata are Ethean-named.
- New code is isolated from legacy modules and all authored source files are at most 2000 lines.
- Production and fixture/Shadow builds are structurally separate.
- Config parsing rejects unknown protocol fields and incompatible profile fingerprints.
- Clock tests cover pre-genesis, boundaries, missed ticks, catch-up, rollback, and deterministic injection.
- CI fails on pin drift, mutable fixture sources, generated-code drift, and empty test discovery.

## Phase 02 acceptance

### SSZ

- Canonical encode, decode, round-trip, and hash-tree-root vectors pass for every selected primitive and container.
- Zero, typical, maximum, malformed offset, trailing-byte, limit-overflow, bitlist delimiter, union, and list/vector cases pass.
- Unknown mandatory fixture types fail.
- JSON or API serialization cannot determine consensus identity.

### Signatures and proofs

- Cross-client vectors pass for proposal and attestation signatures with role-separated keys.
- Wrong key, role swap, wrong slot, wrong message, malformed encoding, expired key, and parameter mismatch fail.
- Type-1 and Type-2 proofs verify exact messages, slots, ordered key groups, participants, overlap rules, and proof generation.
- Invalid child proof, duplicate participant, participant mismatch, wrong component order, oversized proof, and mixed generation fail before expensive work where possible.
- Production binaries cannot activate fake verification.

### Signer safety

- Atomic durable state prevents conflicting signatures after concurrent duties, crash, database rollback, copied backups, and restart.
- Key preparation, exhaustion, rotation, and role independence have boundary tests.
- Secrets are not logged and key files enforce the approved protection policy.

## Phase 03 acceptance

- Genesis, empty/skipped slots, first block, deep chain, wrong proposer/parent/state root, and maximum block fixtures pass.
- Attestation topology, temporal validity, source/target/head ancestry, and duplicate/equivocation behavior match fixtures.
- Fork-choice ticks, pending/known promotion, safe target, head tie-break, side branches, reorgs, and pruning pass.
- Justification/finality boundary cases pass for odd/even validator counts and mid-block updates.
- Consensus arithmetic is checked and deterministic.
- Duty tests cover proposer, attester, aggregator, missed/late work, stale head, reorg during build, and multiple validators per process.

## Phase 04 acceptance

### Wire interoperability

- Ethean connects over the pinned transport to at least two independent peer clients.
- Status, block gossip, attestation subnet gossip, aggregation gossip, blocks-by-root, and blocks-by-range work in both directions.
- Exact topic, message-ID, raw/framed Snappy, SSZ bytes, response codes, stream termination, and limits match fixtures.
- Profile mismatch, malformed frame, decompression bomb, invalid topic/type, oversized response, partial stream, and slow peer fail safely.

### Sync and persistence

- Unknown-parent recovery, out-of-order import, range catch-up, checkpoint bootstrap, backfill, partition/heal, and finality pruning pass.
- Checkpoint acceptance enforces the Phase 00 trust policy and state/block/root/network consistency.
- Acknowledged commits satisfy the declared durability boundary.
- Kill tests cover block/state/index commit, signer update, checkpoint install, pruning, and proof work.
- Restart reconstructs head, checkpoints, fork tree, votes/pools according to policy, network fingerprint, and signer state.
- Corrupt, truncated, foreign-version, disk-full, read-only, and missing-record cases fail with actionable errors.

## Phase 05 acceptance

- CLI help, generated configuration, examples, and deployment commands launch the built binary in CI.
- API payloads are bounded, typed, fuzzed, and separated into public, operator, and test surfaces.
- Admin, profiling, and test-driver endpoints are disabled by default and cannot bind publicly without explicit controls.
- Metrics follow the pinned metrics profile, have bounded labels, stable units, and duty-relevant histograms.
- Logs include build pins, validation outcomes, sync/reorg/finality, proof timing, key exhaustion, and recovery without secret/proof-witness leakage.
- Benchmarks measure canonical operations on declared hardware and do not use BLS or fake proofs as Lean performance evidence.
- Container images run non-root, expose the correct UDP/API/metrics ports, handle shutdown, and pin build/runtime inputs.

## Phase 06 acceptance

- Complete pinned fixture suite: zero failures, zero unknowns, zero silent skips.
- Ordered mixed-client matrix covers connect, status, all gossip families, ancestor recovery, genesis/checkpoint sync, reorg, finality, and restart.
- Mixed-version negative matrix rejects mismatched schema, crypto generation, proof revision, network identity, slot timing, and fixtures.
- Adversarial suite covers malformed SSZ/Snappy, invalid expensive proofs, participant explosion, overlap, unknown-parent flood, duplicate gossip, RPC slowloris, ENR abuse, deep skipped slots, and hostile API JSON.
- Long-run suite covers key lifetime boundaries, sustained proof load, partitions, churn, clock skew, disk pressure, finality stall/recovery, and leak detection.
- Release includes source commit, image digest, SBOM, vulnerability results, compatibility ledger, fixture digest, supported profile, and known limitations.
- Repository scan reports no legacy identities or forbidden protocol paths.
- Every path in `LEGACY_COMPONENT_MATRIX.md` is deleted or superseded and then deleted.

## Non-evidence

The following do not satisfy a gate by themselves:

- successful compilation;
- unit tests that assert only shape or non-empty output;
- a self-only local devnet;
- peer-client majority behavior;
- a README claim;
- a mutable `latest` fixture download;
- a fake/Shadow proof;
- an in-memory database test;
- a benchmark without pins, hardware, inputs, and correctness checks.
