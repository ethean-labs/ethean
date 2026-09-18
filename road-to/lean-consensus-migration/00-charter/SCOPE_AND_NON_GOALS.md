# Scope and Non-Goals

## In scope

### Product identity and build

- Replace package, crate, binary, module, log, CLI, API, metric, documentation, and artifact identity with **Ethean Lean Consensus Client**.
- Create a reproducible Rust workspace and production binary whose version output embeds all compatibility-ledger pins.
- Replace dependency and feature selection so production cannot compile or invoke fake cryptography.

### Protocol profile

- Follow `leanSpec` `main` through exact per-phase commits and fixture digests.
- Freeze all consensus, cryptography, wire, network, genesis, and metrics inputs in one compatibility ledger.
- Reject startup and persisted data when the selected profile is incompatible.

### Consensus and types

- Implement bounded SSZ containers, canonical encoding/decoding, hash-tree roots, signed-object roots, and negative decoding.
- Implement the pinned genesis, clock, slot processing, block transition, attestation validity, vote activation, fork choice, safe target, justification, finality, and pruning semantics.
- Implement proposer, attester, and independent aggregator duties for multiple validators per node.

### Post-quantum cryptography

- Implement separate attestation and proposal XMSS keys.
- Implement exact signing payloads, lifetime behavior, role separation, key preparation, exhaustion, rotation, and crash-safe duplicate-sign prevention.
- Implement the selected individual signature, Type-1 same-message aggregation, recursive merge, participant checks, and Type-2 block proof.
- Enforce proof CPU, memory, concurrency, cancellation, and publication deadlines.

### Networking and synchronization

- Implement the pinned QUIC/libp2p profile, identity, topic grammar, Gossipsub behavior, SSZ/Snappy codec, message ID, status, blocks-by-root, and blocks-by-range.
- Implement bounded semantic validation, pending ancestry, peer/request accounting, recovery, backfill, and checkpoint bootstrap.
- Implement discovery only when the selected phase profile defines its interop contract.

### Persistence and recovery

- Implement typed SSZ storage with schema/network fingerprinting.
- Define atomic block/post-state/index/checkpoint/signer commit boundaries.
- Implement restart reconstruction, corruption handling, pruning, checkpoint import/export, and crash-injection tests.

### Operations

- Implement secure-by-default CLI, local/operator configuration, node and validator APIs, metrics, logs, events, examples, benchmarks, and deployment artifacts.
- Isolate admin, profiling, fixture, Hive, and Shadow surfaces from production.

### Validation and release

- Run complete pinned SSZ, state-transition, fork-choice, signature/proof, slot-clock, sync, and networking fixtures.
- Run mixed-client QUIC interoperability, mismatched-version rejection, recovery, adversarial, long-running, and resource-budget tests.
- Delete all legacy source, docs, examples, configs, tests, benchmarks, and deployment claims after replacements pass.

## Explicit non-goals

### No preservation of the current implementation

- No existing source file receives retain status.
- No JSON consensus root, Serde wire container, BLS flow, Beacon type, local pseudo-Poseidon, local WOTS parameter set, mock network path, no-op storage path, or placeholder API survives.
- No compatibility facade keeps `panro`, `PanroClient`, legacy module names, or old database records alive.
- No line-by-line port or peer-client crate-layout copy is planned.

### No legacy Beacon compatibility

Unless a later Lean specification explicitly requires a separately scoped adapter, this migration does not implement:

- Beacon REST compatibility;
- BLS validator keys or BLS aggregation;
- 12-second slots or 32-slot epochs;
- Beacon committees, sync committees, RANDAO, proposer boost, Casper FFG, or Beacon LMD-GHOST constants;
- deposits, withdrawals, exits, slashing economics, rewards, effective balances, or 32 ETH assumptions;
- `/eth2/` gossip topics, Beacon fork digests, weak-subjectivity formulas, or execution-payload validity semantics.

### No execution-client implementation

Ethean remains a consensus client. Engine API or execution payload work is excluded until a frozen Lean phase explicitly includes it. Ream and Gean execution experiments are not authority for this migration.

### No speculative roadmap implementation

The program does not pre-implement Goldfish, PQ heartbeat, Gossipsub v2/set reconciliation, APS, rainbow staking, one-ETH validator economics, STF proofs, or new proof gossip before those surfaces are part of the selected `leanSpec`/devnet pin.

Research direction informs boundaries, not current wire behavior.

### No production claims from simulation

Shadow fake proofs, deterministic keys, mock clocks, in-memory storage, localhost self-tests, and synthetic benchmarks cannot satisfy production security, interop, durability, or performance gates.

### No majority-vote protocol decisions

The seven peer clients do not define constants or formats by consensus. Their audited snapshots span devnet-2, devnet-4, and devnet-5-shaped behavior, multiple XMSS generations, 8/16 attestation limits, 32/52-byte keys, 1,208/2,536/3,112-byte signatures, and differing discovery/framing behavior.

### No silent unknowns

Unknown protocol values are Phase 00 blockers. They are not deferred as generic TODOs, configured per node, inferred from filenames, or selected from library defaults.

### No unrelated cleanup

This planning set does not authorize editing or resolving unrelated working-tree deletions/moves, the existing root documentation, source code, or the pre-existing `03-architecture/README.md`.

## Scope change rule

A proposed addition enters scope only with:

1. an authoritative pinned source;
2. an identified owner phase;
3. concrete acceptance and negative tests;
4. resource and security impact;
5. explicit effect on the removal matrix.

Without those items, it remains out of scope.
