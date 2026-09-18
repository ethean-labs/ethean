# Dependency Rules

## Directed acyclic graph

An arrow means “may depend on.” Transitive dependencies are omitted.

```text
ethean-primitives

ethean-profile      -> ethean-primitives
ethean-types        -> ethean-primitives, ethean-profile
ethean-crypto       -> ethean-primitives, ethean-profile, ethean-types
ethean-transition   -> ethean-primitives, ethean-profile, ethean-types, ethean-crypto
ethean-fork-choice  -> ethean-primitives, ethean-profile, ethean-types
ethean-storage      -> ethean-primitives, ethean-profile, ethean-types
ethean-network-wire -> ethean-primitives, ethean-profile, ethean-types

ethean-network      -> ethean-primitives, ethean-profile, ethean-types,
                       ethean-network-wire, ethean-metrics
ethean-sync         -> ethean-primitives, ethean-profile, ethean-types,
                       ethean-crypto, ethean-transition, ethean-network-wire,
                       ethean-network, ethean-metrics
ethean-validator    -> ethean-primitives, ethean-profile, ethean-types,
                       ethean-crypto, ethean-node, ethean-metrics
ethean-node         -> ethean-primitives, ethean-profile, ethean-types,
                       ethean-crypto, ethean-transition, ethean-fork-choice,
                       ethean-storage, ethean-network-wire, ethean-metrics
ethean-rpc          -> ethean-primitives, ethean-profile, ethean-types,
                       ethean-node, ethean-validator, ethean-metrics
ethean-metrics      -> ethean-primitives

ethean              -> every library crate required for composition
```

The textual list above is normative except for one ordering correction required to keep the graph acyclic: `ethean-validator` may depend on the stable node handle API, while `ethean-node` must not depend on `ethean-validator`. Validator actions enter through node commands. Node events and snapshots leave through node subscriptions.

## Layer rules

### Foundation layer

`primitives`, `profile`, and `types` are stable vocabulary. They cannot depend on application services, runtime frameworks, storage engines, RPC frameworks, or transport libraries.

### Pure core layer

`crypto`, `transition`, and `fork-choice` implement deterministic protocol logic. They may allocate and parallelize pure computation, but they may not perform I/O or spawn long-running tasks.

Forbidden direct dependencies include:

- async runtimes and task schedulers;
- filesystem, socket, HTTP, QUIC, or database clients;
- process environment and command-line parsers;
- global logging/exporter initialization;
- system clocks, random generators, or thread-local mutable state in decision paths.

Time, randomness, profiles, and verification inputs must be explicit parameters. Cryptographic key generation may accept a caller-provided cryptographically secure RNG trait; verification remains deterministic.

### Adapter layer

`storage`, `network-wire`, `network`, `rpc`, and `metrics` translate between the outside world and domain interfaces. Adapters must not contain consensus policy.

### Orchestration layer

`sync`, `validator`, and `node` coordinate workflows through typed commands and events. Only `node` owns mutable canonical chain state. `sync` and `validator` are clients of the node handle.

### Composition layer

`bin/ethean` chooses concrete adapters and starts services. No library crate may depend on the executable package.

## Explicitly forbidden edges

- Any dependency from a lower layer to `node`, `sync`, `validator`, `rpc`, or the executable, except the documented validator-to-node handle edge.
- `types -> crypto`; domain objects store signature bytes or typed wrappers but do not invoke verification.
- `transition -> storage|network|network-wire|rpc|node|sync|validator|metrics`.
- `fork-choice -> storage|network|network-wire|rpc|node|sync|validator|metrics|transition`.
- `storage -> transition|fork-choice|network|node|sync|validator|rpc`.
- `network-wire -> network`; schemas cannot depend on a transport implementation.
- `network -> storage|transition|fork-choice|node|sync|validator|rpc`.
- `sync -> storage|fork-choice|node|validator|rpc`; node interaction uses a handle contract supplied at construction.
- `rpc -> storage|transition|fork-choice|network`; queries use service handles.
- Any pair of crates depending on each other, including through optional features, dev-dependencies, build-dependencies, examples, or tests.

## Ownership and communication

- Cross-service communication uses bounded channels with typed commands and events.
- Commands that can change chain state carry a request identifier and return a typed receipt.
- Read consumers use immutable `Arc` snapshots or owned view models. A snapshot cannot expose interior mutability.
- Traits live with their consumer. For example, a node persistence port lives in `ethean-node`; `ethean-storage` supplies the adapter in the composition root.
- Shared mutable state behind a workspace-wide lock is prohibited.
- Storage transactions cannot call back into node logic.
- Metrics and tracing are side effects after a decision; their failure cannot alter that decision.

## Feature rules

- Features may select implementations, not erase architectural boundaries.
- No feature may introduce a forbidden edge or combine two target crates.
- Backend features belong to adapter crates.
- Protocol-version selection is profile data, not a compile-time feature.
- Test utilities are exposed from dedicated `test_utils` modules behind a `test-utils` feature and obey the same DAG.

## Enforcement

CI must:

1. Run `cargo metadata --all-features` and materialize the package graph.
2. Fail on a cycle or an edge absent from an architecture-owned allowlist.
3. Run the check for normal, dev, build, target-specific, and optional dependencies.
4. Reject I/O/runtime dependencies in pure core manifests.
5. Search pure core sources for filesystem, socket, process-environment, database, and runtime imports.
6. Compile each crate independently with its documented minimal features.

Any required dependency reversal is implemented with commands, events, snapshots, consumer-owned traits, or a lower-level value type. It is never solved with a mutual dependency.
