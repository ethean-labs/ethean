# Target Architecture

This directory defines the workspace architecture that the Lean Consensus migration must produce. It is normative for crate boundaries, ownership, dependencies, data movement, and source-file size. Later implementation phases may refine protocol details, but they must not weaken these boundaries without first updating the architecture plan.

## Documents

- [TARGET_WORKSPACE.md](TARGET_WORKSPACE.md) defines every target crate and its concrete modules.
- [DEPENDENCY_RULES.md](DEPENDENCY_RULES.md) defines the permitted dependency graph and forbidden edges.
- [DATA_FLOW.md](DATA_FLOW.md) defines chain import, validator duty, network, and storage flows.
- [MODULE_SIZE_POLICY.md](MODULE_SIZE_POLICY.md) defines the 300-line source limit and required decomposition process.

## Non-negotiable decisions

1. `ethean-node` is the only owner of mutable canonical chain state.
2. `ethean-transition` and `ethean-fork-choice` are deterministic, I/O-free core crates.
3. Wire representations are isolated in `ethean-network-wire`; domain types do not depend on networking.
4. Storage exposes durable records and atomic write batches, but never selects the canonical head.
5. Network code transports validated envelopes and emits events; it never mutates chain state.
6. Validator code proposes actions through node commands; it never writes state or storage directly.
7. Dependencies form a directed acyclic graph. Mutual crate dependencies are prohibited.
8. Every hand-written Rust source file is at most 300 physical lines, including tests embedded in that file.

## Target layout

```text
Cargo.toml
crates/
  primitives/
  profile/
  types/
  crypto/
  transition/
  fork-choice/
  storage/
  network-wire/
  network/
  sync/
  validator/
  node/
  rpc/
  metrics/
bin/
  ethean/
```

Each listed directory is a workspace package with its own `Cargo.toml`, `README.md`, and `src/` tree. Package names use the `ethean-` prefix, while folder names remain short. The executable package and binary are both named `ethean`.

## Architectural acceptance criteria

The architecture phase is complete only when:

- `cargo metadata` reports exactly the intended workspace members and no historical root package;
- a dependency-graph check reports no cycle and no forbidden edge;
- all mutable canonical-state writes are reachable only through the node chain-state owner;
- core crates compile without async runtimes, filesystem, database, socket, HTTP, or tracing-subscriber dependencies;
- wire decoding is bounded and separated from semantic validation;
- storage commits state, blocks, fork-choice metadata, and import markers atomically;
- every code directory has an English `README.md`;
- source-size verification rejects any hand-written source above 300 lines; and
- repository-wide legacy-name scans pass under the retirement policy.
