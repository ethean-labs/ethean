# Phase 09 — Validator and Node Duties

## Pinned inputs

- Snapshot `LC-D5-2026-09-19`; baseline `880982f`; `leanSpec` `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54`; fixture SHA-256 manifest from Phase 00.
- `leanSig` `15cbdd43ec8525aa43fea2f42cafc5ed366084ae`; `leanVM` `e2592df4e30fdddbbf8ae26a333116c68cec7026`.
- Timing profile: 4,000 ms slot divided into five 800 ms intervals. Interval semantics and all tolerated lateness are profile data, not hard-coded defaults.
- Registry/profile: 4,096 validators, separate role keys, eight unique attestation-data values, Type-1 aggregate gossip, Type-2 block proof.
- Rust `1.97.1`, lockfile and build image are fixed. Peer schedule evidence: Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, and gean `b78f6d737f4df57a72d5e230635681235fda8024`.

## Objective

Build deterministic proposer, attester, and aggregator duties around one chain-state owner, bounded typed queues, explicit deadlines, and safe suppression when the node lacks a trustworthy current view.

## Non-goals

- No Beacon committee shuffle, RANDAO, selection proof, epoch subscription, rewards, balances, slashing workflow, execution payload, builder market, or proposer-equals-aggregator assumption.
- No protocol invention for unspecified failover. Deployment policy is labeled separately from consensus behavior.
- No late publication merely to maximize participation.

## Entry criteria

- Phases 00–08 pass, including deterministic clock/fork choice, durable signer reservations, and isolated proof workers.
- Production functions for head, source, target, proposer selection, subnet, block transition, and proof verification match fixtures.
- Sync state exposes trustworthy lag and peer-horizon data; queue/resource budgets are approved.
- Aggregator coverage configuration is validated before duties start.

## Exact old and new paths

Replace then delete:

- `src/client.rs`, `src/main.rs`, `src/consensus/validator_management.rs`, `src/consensus/block_processing.rs`, `src/consensus/attestation_processing.rs`.
- `src/integration/coordinator.rs`, `src/integration/bridge.rs`, `src/integration/conflict_resolver.rs`, and placeholder validator routes in `src/api/validator.rs`.

Create:

- `crates/ethean-node/src/{lib.rs,chain_owner.rs,commands.rs,events.rs,shutdown.rs}`.
- `crates/ethean-validator/src/{lib.rs,scheduler.rs,proposer.rs,attester.rs,aggregator.rs,duty_gate.rs}`.
- `crates/ethean-node/src/block_builder/{mod.rs,selection.rs,transition.rs,publish.rs}`.
- `bin/ethean/src/main.rs`, `tests/integration/duties.rs`, `tests/recovery/duty_restart.rs`, and `tests/interop/duties.rs`.

All new code directories require English `README.md` files and all hand-written source files stay within 300 lines.

## Ordered tasks

1. Make `ChainOwner` the only writer of transition state, fork choice, checkpoints, pools, and import status. Give network, API, storage, signer, and prover typed bounded command/event channels.
2. Convert the injected clock into monotonically ordered `(slot, interval, generation)` events; deduplicate repeats and explicitly catch up or skip after pauses.
3. Snapshot immutable chain views for workers. Results carry generation, parent root, slot, and deadline and are rejected if stale.
4. Implement the duty gate: suppress signing while pre-genesis, syncing, missing parent state, profile-mismatched, signer-unsafe, or beyond configured head lag. A stalled network policy cannot silently redefine consensus.
5. Reserve each local duty before work. An in-flight/completed key prevents duplicate proposal, attestation, aggregation, and signer calls.
6. Attester flow: select head/source/target from one snapshot, derive subnet, reserve signer duty, sign with attestation key, verify/import locally, then publish before deadline.
7. Aggregator flow: collect only assigned subnets, verify before admission, select disjoint same-data coverage, dispatch bounded Type-1 work, locally verify, import, and publish at the pinned interval.
8. Proposer flow: refresh parent snapshot, select canonical proof coverage, construct body, run transition to state root, reserve proposal signature, create/verify Type-2 proof, import locally, then publish at slot boundary.
9. Recheck parent/head and wall deadline before every irreversible sign or publish step. Cancel stale proof/build work; never publish after the allowed window.
10. Prioritize proposal and chain-import commands over optional reaggregation. Apply queue-specific backpressure without blocking network or clock tasks.
11. Implement graceful shutdown: stop new duties, cancel workers, finish durable signer/storage boundaries, and emit an incomplete-duty record.
12. Emit stage timings, missed/suppressed reasons, queue depth, stale result, signer/prover outcome, and coverage using bounded labels.

## Legacy deletion obligations

- Delete monolithic `PanroClient`, generic conflict resolver, placeholder templates, simulated consensus loops, and duplicate state owners.
- Remove 12-second sleeps, `slot / 32`, epoch duties, sync committees, BLS mock keys, and Beacon API duty semantics.
- There is one production scheduler and one production chain owner; no legacy mode remains.

## Security and specification risks

- Duplicate tick delivery or worker races can double sign.
- Mixed-time snapshots can produce a valid signature for a stale parent or inconsistent source/target.
- Unbounded channels and prover work can starve proposal/import progress.
- A stale/sync gate may harm liveness if its horizon excludes pending blocks.
- Runtime aggregator toggles can create subnet gaps; role changes require pre-established subscriptions and coverage checks.
- Local import before publication must perform the same validation as remote import.

## Tests

- Fixtures: proposer selection, source/target/head, interval promotion, skipped slots, safe target, exact block build root, and maximum body.
- Negative: duplicate/reordered ticks, cross-role signer request, stale parent, unavailable state, conflicting root, late result, queue full, insufficient aggregator coverage, and profile mismatch.
- Recovery: kill at every duty stage, restart across interval/slot boundaries, clock rollback/forward, corrupt incomplete-duty record, and signer/prover/storage failure.
- Interop: at least two Ethean nodes and one pinned peer exchange proposals, raw attestations, Type-1 aggregates, and Type-2 blocks; compare roots/checkpoints and duty timing.
- Concurrency: deterministic replay under randomized task scheduling, duplicate network delivery, and simultaneous local/remote block import.

## Validation commands

```text
cargo +1.97.1 test -p ethean-validator --locked
cargo +1.97.1 test -p ethean-node --locked
cargo +1.97.1 test --test duties --locked
cargo +1.97.1 test --test duty_restart --locked
cargo +1.97.1 test --test duties --release --locked --features interop
cargo +1.97.1 clippy -p ethean-validator -p ethean-node --all-targets --locked -- -D warnings
```

## Exit criteria

- Deterministic replay yields identical signatures, roots, head, and checkpoints.
- Duplicate triggers cannot create a second reservation or signature.
- Duties are suppressed or published according to explicit pinned deadlines; no stale result mutates chain state.
- A 24-hour 4-second-slot run has no queue growth, duplicate duty, deadlock, or process-liveness-only false health.
- Old client/integration/duty paths are deleted and touched source files are at most 300 lines.

## Rollback and data policy

Duty caches are reconstructable; signer journals and accepted chain data are not rolled back. On binary rollback, cancel incomplete jobs, revalidate profile/schema, rebuild scheduling from wall slot, and preserve all signer reservations. Never replay an old duty solely because its completion event was lost.

## Evidence artifacts

- Timing diagram, command ownership map, deterministic replay hash, duplicate-trigger report, duty suppression matrix, 24-hour timeline, mixed-client roots/checkpoints, and queue/resource traces in `artifacts/phase-09/`.

## Dependencies

Depends on Phases 04–08. Phase 10 supplies publication and retrieval, Phase 11 supplies restart/sync durability, and Phases 12–13 qualify fleet behavior.
