# Target Rust Workspace

## Workspace root

The root `Cargo.toml` becomes a virtual workspace manifest. It contains `[workspace]`, shared package metadata, shared dependency versions, lint policy, and release profiles. It contains no `[package]`, library target, binary target, or application dependencies.

Workspace members, in build-layer order:

```toml
members = [
  "crates/primitives",
  "crates/profile",
  "crates/types",
  "crates/crypto",
  "crates/transition",
  "crates/fork-choice",
  "crates/storage",
  "crates/network-wire",
  "crates/network",
  "crates/sync",
  "crates/validator",
  "crates/node",
  "crates/rpc",
  "crates/metrics",
  "bin/ethean",
]
resolver = "2"
```

## Crate specifications

### `ethean-primitives` — `crates/primitives`

Small, stable value types with no protocol orchestration.

Target files:

- `src/lib.rs`: module declarations and deliberate public re-exports only.
- `src/hash.rs`: `Hash32`, zero value, parsing, formatting, and fixed-size conversion.
- `src/slot.rs`: `Slot`, checked arithmetic, and ordering.
- `src/epoch.rs`: `Epoch` and slot/epoch conversion using explicit parameters.
- `src/validator.rs`: `ValidatorIndex` and compact identifiers.
- `src/bytes.rs`: bounded byte containers used by domain and wire layers.
- `src/error.rs`: primitive conversion errors.

It must not contain chain configuration, signatures, state, serialization policy, clocks, or I/O.

### `ethean-profile` — `crates/profile`

Resolved network and protocol profile values.

Target files:

- `src/lib.rs`: public profile surface.
- `src/profile.rs`: immutable `ChainProfile`.
- `src/preset.rs`: named built-in presets.
- `src/fork.rs`: fork/version schedule and activation lookup.
- `src/limits.rs`: message, collection, and resource bounds.
- `src/validation.rs`: cross-field profile validation.
- `src/error.rs`: profile errors.

Profile loading from files or environment belongs to the executable. This crate accepts already parsed values and validates them without I/O.

### `ethean-types` — `crates/types`

Consensus-domain structures and canonical containers.

Target modules:

- `src/block/{mod.rs,header.rs,body.rs,signed.rs}`.
- `src/state/{mod.rs,chain_state.rs,validator.rs,checkpoint.rs}`.
- `src/operation/{mod.rs,attestation.rs,aggregate.rs,slashing.rs}`.
- `src/signing/{mod.rs,message.rs,domain.rs}`.
- `src/receipt/{mod.rs,import.rs,duty.rs}`.
- `src/error.rs` and `src/lib.rs`.

Types express invariants through constructors and bounded collections. They contain no database keys, network topics, transport peer IDs, RPC response wrappers, or runtime handles.

### `ethean-crypto` — `crates/crypto`

Post-quantum signing, verification, hashing, and aggregate-proof interfaces.

Target modules:

- `src/hash/{mod.rs,digest.rs,merkle.rs}`.
- `src/signature/{mod.rs,public_key.rs,signature.rs,verify.rs}`.
- `src/aggregate/{mod.rs,proof.rs,verify.rs}`.
- `src/domain.rs`, `src/backend.rs`, `src/error.rs`, and `src/lib.rs`.

Public verification functions are deterministic and receive all inputs explicitly. Secret-key persistence, remote signer transport, and validator key lifecycle are outside this crate.

### `ethean-transition` — `crates/transition`

Pure state-transition and semantic validation logic.

Target modules:

- `src/block/{mod.rs,validate.rs,apply.rs}`.
- `src/operation/{mod.rs,attestation.rs,aggregate.rs,slashing.rs}`.
- `src/slot/{mod.rs,process.rs}`.
- `src/epoch/{mod.rs,process.rs,rewards.rs,registry.rs}`.
- `src/context.rs`, `src/outcome.rs`, `src/error.rs`, and `src/lib.rs`.

The main API consumes an immutable pre-state plus an explicit transition context and returns a new state and `TransitionOutcome`. It does not access time, disk, network, global configuration, or mutable singletons.

### `ethean-fork-choice` — `crates/fork-choice`

Pure fork-choice store updates and head selection.

Target modules:

- `src/store/{mod.rs,state.rs,checkpoint.rs}`.
- `src/update/{mod.rs,block.rs,vote.rs,tick.rs}`.
- `src/head/{mod.rs,select.rs,tie_break.rs}`.
- `src/prune.rs`, `src/error.rs`, and `src/lib.rs`.

The crate returns updated fork-choice data and decisions. It neither persists data nor owns the canonical chain state.

### `ethean-storage` — `crates/storage`

Durable persistence contracts and backend implementations.

Target modules:

- `src/api/{mod.rs,reader.rs,writer.rs,transaction.rs}`.
- `src/record/{mod.rs,block.rs,state.rs,fork_choice.rs,metadata.rs}`.
- `src/key/{mod.rs,schema.rs}`.
- `src/backend/{mod.rs,memory.rs,rocks.rs}`.
- `src/migration/{mod.rs,version.rs,runner.rs}`.
- `src/error.rs` and `src/lib.rs`.

Writes use an atomic `StorageBatch`. The crate stores node decisions but does not make them. Backend-specific types remain private.

### `ethean-network-wire` — `crates/network-wire`

Versioned protocol envelopes, bounded codecs, topic identities, and request/response schemas.

Target modules:

- `src/gossip/{mod.rs,topic.rs,envelope.rs,codec.rs}`.
- `src/request/{mod.rs,method.rs,codec.rs}`.
- `src/response/{mod.rs,status.rs,codec.rs}`.
- `src/convert/{mod.rs,to_domain.rs,from_domain.rs}`.
- `src/version.rs`, `src/limits.rs`, `src/error.rs`, and `src/lib.rs`.

Decoding verifies lengths, discriminants, and canonical encoding. Semantic consensus validation remains in core crates.

### `ethean-network` — `crates/network`

Transport runtime, peer lifecycle, discovery, gossip, request/response routing, and resource control.

Target modules:

- `src/service/{mod.rs,handle.rs,event.rs,command.rs}`.
- `src/peer/{mod.rs,id.rs,manager.rs,score.rs}`.
- `src/gossip/{mod.rs,router.rs,validation.rs}`.
- `src/request_response/{mod.rs,router.rs,pending.rs}`.
- `src/discovery/{mod.rs,service.rs}`.
- `src/limits/{mod.rs,rate.rs,connection.rs}`.
- `src/config.rs`, `src/error.rs`, and `src/lib.rs`.

The public boundary is a command sender plus a network-event receiver. It never receives mutable chain state.

### `ethean-sync` — `crates/sync`

Synchronization policy and orchestration state machine.

Target modules:

- `src/service/{mod.rs,handle.rs,event.rs,command.rs}`.
- `src/planner/{mod.rs,range.rs,checkpoint.rs}`.
- `src/download/{mod.rs,blocks.rs,retry.rs}`.
- `src/verify/{mod.rs,batch.rs}`.
- `src/status.rs`, `src/error.rs`, and `src/lib.rs`.

Sync asks the network for data and submits ordered import commands to the node. It does not mutate canonical state or write storage.

### `ethean-validator` — `crates/validator`

Duty planning, proposal construction requests, signing abstraction, and publication decisions.

Target modules:

- `src/service/{mod.rs,handle.rs,event.rs,command.rs}`.
- `src/duty/{mod.rs,schedule.rs,attest.rs,propose.rs,aggregate.rs}`.
- `src/signer/{mod.rs,trait.rs,local.rs,remote.rs}`.
- `src/slashing/{mod.rs,protection.rs,record.rs}`.
- `src/config.rs`, `src/error.rs`, and `src/lib.rs`.

The validator receives immutable node snapshots and duty notifications. Signed outputs return to the node for validation and publication.

### `ethean-node` — `crates/node`

Application-domain coordinator and sole owner of mutable canonical chain state.

Target modules:

- `src/service/{mod.rs,handle.rs,command.rs,event.rs,runner.rs}`.
- `src/chain/{mod.rs,owner.rs,snapshot.rs,import.rs,head.rs}`.
- `src/pipeline/{mod.rs,block.rs,operation.rs}`.
- `src/persistence/{mod.rs,commit.rs,recovery.rs}`.
- `src/publication/{mod.rs,gossip.rs}`.
- `src/config.rs`, `src/error.rs`, and `src/lib.rs`.

`chain::owner::ChainStateOwner` owns the canonical `ChainState`, fork-choice store, finalized checkpoint, and current head. It runs in one serialized command loop. Other crates receive snapshots, receipts, or commands, never `&mut ChainState`.

### `ethean-rpc` — `crates/rpc`

HTTP/WebSocket boundary, request validation, response mapping, and subscriptions.

Target modules:

- `src/server/{mod.rs,run.rs,state.rs}`.
- `src/routes/{mod.rs,node.rs,chain.rs,validator.rs}`.
- `src/ws/{mod.rs,subscription.rs}`.
- `src/types/{mod.rs,request.rs,response.rs}`.
- `src/error.rs`, `src/config.rs`, and `src/lib.rs`.

RPC calls node and validator handles. It never reads databases or holds consensus state directly.

### `ethean-metrics` — `crates/metrics`

Stable metric names, labels, recorders, and exporter startup.

Target modules:

- `src/record/{mod.rs,chain.rs,network.rs,storage.rs,validator.rs}`.
- `src/export/{mod.rs,prometheus.rs}`.
- `src/registry.rs`, `src/config.rs`, `src/error.rs`, and `src/lib.rs`.

Metric recording is observational. Consensus decisions must not branch on recorder success or metric values.

### `ethean` — `bin/ethean`

Composition root and only production executable.

Target modules:

- `src/main.rs`: process entry and exit status.
- `src/cli.rs`: command-line schema.
- `src/config/{mod.rs,file.rs,environment.rs,resolve.rs}`.
- `src/runtime/{mod.rs,build.rs,shutdown.rs}`.
- `src/commands/{mod.rs,node.rs,validator.rs,database.rs}`.
- `src/wiring.rs`: constructs services and channels.
- `src/error.rs`.

Only this package may assemble concrete backends, initialize tracing, read process configuration, install signal handlers, and start all long-running services.

## Public API discipline

Every crate has a narrow `lib.rs` that re-exports supported interfaces. Cross-crate callers must not import private backend modules. Concrete backend structs, transport behavior types, and server framework types stay behind crate-owned handles or traits. New shared abstractions go into the lowest semantically correct crate; a generic `common` crate is prohibited.
