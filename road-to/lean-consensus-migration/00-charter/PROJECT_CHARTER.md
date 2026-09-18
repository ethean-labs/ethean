# Project Charter

## Mission

Replace the current Beacon-era Panro prototype with a clean, interoperable, security-gated Rust implementation named **Ethean Lean Consensus Client**.

The delivered client is a Lean Consensus layer client. It is not an execution client, a legacy Beacon compatibility implementation, or a generic blockchain node.

## Why replacement is required

The audited baseline is incompatible with Lean interoperability:

- package and binary identity remain `panro`;
- consensus containers are hand-written Serde structures;
- consensus roots are SHA-256 over JSON rather than SSZ hash-tree roots;
- BLS, Beacon validator lifecycle, balances, epochs, committee assumptions, and execution payload types are embedded in active paths;
- local WOTS and pseudo-Poseidon code do not implement a pinned Lean cryptographic construction;
- networking is primarily mock/in-memory and lacks the pinned Lean QUIC, gossip, SSZ/Snappy, and request/response wire contract;
- the production-named RocksDB backend acknowledges writes without persisting data;
- APIs, examples, benchmarks, and documentation advertise behavior that the code does not provide;
- no pinned leanSpec conformance or mixed-client interoperability gate exists.

Compiling the old tree does not establish protocol correctness.

## Migration decision

All old implementation surfaces receive exactly one disposition:

1. **replace-then-delete** — a responsibility is required, but a new implementation must pass its acceptance gate before the old path is removed;
2. **rename-by-replacement** — an identity-bearing path is replaced by a newly authored Ethean path and then deleted;
3. **delete** — the surface is obsolete, misleading, empty, duplicated, or outside Lean scope.

There is no retain status. No old file becomes part of the final client merely because its framework, trait, or module boundary looks generic.

## Protocol authority

Development follows `leanSpec` `main`, but each implementation phase freezes exact inputs before code is accepted:

- `leanSpec` commit and fixture bundle SHA-256;
- pq-devnet/network profile and genesis assets;
- `leanSig`, `leanVM` or `leanMultisig` revisions;
- full XMSS, hash, proof, and serialization parameters;
- networking and metrics revisions;
- toolchain, dependency, and generated-code provenance.

Upstream `main` remains the direction of travel. A frozen phase is not silently rebased. A later upstream change enters through an explicit compatibility-ledger update and migration gate.

## Phase ownership

- **Phase 00 — Protocol and Security Owner:** freezes authoritative pins, schemas, limits, trust assumptions, and threat model.
- **Phase 01 — Platform Owner:** creates the Ethean workspace, product identity, profile loader, clock, feature isolation, and conformance harness.
- **Phase 02 — Types and Cryptography Owner:** owns SSZ, roots, XMSS roles, signer durability, and recursive proof boundaries.
- **Phase 03 — Consensus Owner:** owns genesis, transition, fork choice, finality, pools, and duty semantics.
- **Phase 04 — Network and Persistence Owner:** owns transport, codecs, sync, checkpointing, storage, restart, and pruning.
- **Phase 05 — Operations Owner:** owns CLI, API, metrics, logging, benchmarks, examples, and deployment.
- **Phase 06 — Interoperability and Release Owner:** owns complete conformance, mixed-client tests, adversarial/recovery/long-run gates, reproducibility, and legacy removal.

An owner phase may define new files before deleting old ones, but cannot declare completion while a legacy import or fallback remains reachable.

## Required architecture boundaries

The replacement must maintain explicit, testable boundaries for:

- immutable versioned consensus profiles;
- canonical bounded SSZ types and hash-tree roots;
- pure state transition;
- deterministic fork choice and finality;
- dual-role stateful signing;
- Type-1 and Type-2 proof formats without decode heuristics;
- proposer, attester, aggregator, and node roles;
- topic construction, codec, message ID, semantic validation, and publication;
- status, root/range recovery, pending-parent handling, and checkpoint bootstrap;
- typed persistence, atomic updates, reconstruction, and pruning;
- injectable slot/interval clock;
- resource admission and observability;
- production versus fixture/Shadow cryptography.

These are responsibility boundaries, not permission to copy peer layouts.

## Phase 00 mandatory decisions

Phase 00 must resolve with exact evidence:

- current target protocol/devnet generation;
- finality and fork-choice generation, including whether modified 3SF-mini remains active;
- complete SSZ containers, field order, list/vector bounds, roots, and signed-object identity;
- 8-versus-16 `MAX_ATTESTATION_DATA`;
- Type-1 versus Type-2 block envelope, component ordering, and proof limits;
- public-key/signature sizes and the exact XMSS instantiation;
- slot-to-XMSS lifetime index binding, activation, exhaustion, rotation, and rollback behavior;
- durable duplicate-signing and role separation;
- fork/topic identity, Gossipsub profile, message-ID byte order, Snappy framing, payload limits, and request/response semantics;
- slot cadence and interval actions;
- checkpoint trust anchor and canonical-membership assurance;
- fixture provenance, mandatory type coverage, and allowed test-only proof behavior;
- target CPU, RAM, proof deadline, cancellation, and long-run budgets.

The result must be machine-readable and human-reviewed. “Use current defaults” is not an accepted decision.

## Security stop conditions

Implementation or release stops if any of these is true:

- cryptographic revisions or parameters are ambiguous;
- production can activate fake verification or deterministic devnet keys;
- signer state can be reused after crash, rollback, concurrency, or backup restore;
- untrusted decode or proof work has no enforced resource bound;
- checkpoint acceptance has no explicit trust policy;
- storage acknowledges a consensus update without the promised durability;
- fixture code and protocol pins can drift;
- mandatory fixture classes are skipped or unknown;
- public admin/test surfaces are unauthenticated or enabled by default;
- protocol-profile mismatch can continue rather than fail closed.

## Change governance

Every phase change must include:

1. compatibility-ledger revision;
2. affected-path inventory;
3. acceptance evidence;
4. migration/recovery impact;
5. explicit legacy deletions;
6. an English development summary under the project documentation conventions.

Peer evidence may motivate a design, but acceptance cites the frozen authority and executable tests.

## Completion statement

The charter is fulfilled only when the Ethean-named replacement passes Phase 06 and every baseline implementation surface has been deleted or replaced, with no reachable Panro/Beacon/BLS/JSON-root/placeholder compatibility path.
