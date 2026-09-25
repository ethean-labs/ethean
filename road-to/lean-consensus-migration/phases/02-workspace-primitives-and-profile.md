# Phase 02: Workspace, Primitives, and Profile

## Pinned inputs

- Accepted `spec/pins/phase-02.lock.toml`.
- Exact primitive aliases, integer widths, root sizes, domain types, profile fields, and bounds extracted from the Phase 00 leanSpec snapshot.
- Rust toolchain and dependency versions recorded in the lock; Cargo dependencies use exact lockfile resolutions.

## Objective

Convert the root package into a workspace and establish one Lean primitive/profile authority. Move the executable into `ethean-node`; all surviving modules consume the new primitives without duplicate aliases or hard-coded protocol constants.

This phase lays the workspace foundation described in [../03-architecture/TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md). Later phases add the remaining members without renaming the plan: `ethean-primitives`, `ethean-profile`, `ethean-types`, `ethean-crypto`, `ethean-transition`, `ethean-fork-choice`, `ethean-storage`, `ethean-network-wire`, `ethean-network`, `ethean-sync`, `ethean-validator`, `ethean-node`, `ethean-rpc`, `ethean-metrics`, and `bin/ethean`. Phase 02 creates and wires only the primitives, profile, and node shell; the other crates appear in their owning phases.

## Non-goals

- No canonical container SSZ implementation, genesis construction, transition, fork choice, or finality.
- No generic multi-network abstraction beyond fields present in the pinned profile.
- No compatibility re-export of old Beacon names.

## Entry criteria

- Phase 01 leaves one `ethean` identity.
- Phase 00 has resolved every primitive and profile field required here.
- A dependency graph proves which old modules use each alias or constant.

## Exact affected old and new paths

- Replace root package manifest: `Cargo.toml` -> workspace-only `Cargo.toml`; regenerate `Cargo.lock`.
- Move executable/library shell: `src/bin/main.rs`, `src/main.rs`, `src/client.rs`, `src/cli.rs`, `src/lib.rs` -> `crates/ethean-node/Cargo.toml`, `crates/ethean-node/src/main.rs`, `crates/ethean-node/src/client.rs`, `crates/ethean-node/src/cli.rs`, `crates/ethean-node/src/lib.rs`.
- Replace aliases: `src/types/block.rs`, `src/types/checkpoint.rs` -> `crates/ethean-primitives/Cargo.toml`, `crates/ethean-primitives/src/lib.rs`, `crates/ethean-primitives/src/numbers.rs`, `crates/ethean-primitives/src/root.rs`.
- Replace configuration authority: `src/config/mod.rs` -> `crates/ethean-profile/Cargo.toml`, `crates/ethean-profile/src/lib.rs`, `crates/ethean-profile/src/profile.rs`, `crates/ethean-profile/src/error.rs`.
- Move untouched legacy modules under node temporarily: `src/api/`, `src/bench/`, `src/consensus/`, `src/crypto/`, `src/integration/`, `src/network/`, `src/optimization/`, `src/storage/`, `src/types/`, `src/utils/` -> matching `crates/ethean-node/src/` paths.
- New tests/evidence: `crates/ethean-primitives/tests/`, `crates/ethean-profile/tests/`, `spec/fixtures/phase-02/manifest.toml`, `docs/lean-consensus-migration-phase-02-workspace.md`.

## Ordered implementation tasks

1. Create workspace manifests and crate READMEs; enforce the repository’s 2000-line source limit.
2. Define only pinned newtypes and root/domain wrappers in `ethean-primitives`; implement checked conversions and arithmetic.
3. Define the immutable protocol profile in `ethean-profile`; parse committed profile data and reject unknown/missing fields.
4. Move node entry points and surviving modules into `ethean-node`, preserving one executable.
5. Replace primitive imports throughout moved code. Remove old `Slot`, `Epoch`, `Root`, `BlockHash`, and validator-index aliases rather than re-exporting them.
6. Route surviving configuration consumers through `Profile`; keep operational API/network settings separate in the node crate.
7. Add compile-fail tests for mixed units and runtime tests for overflow/boundary parsing.
8. Prove the root `src/` tree is gone and the workspace has no dependency cycle.

## Deletion obligations

- Delete root `src/` after all files are moved or replaced.
- Delete `src/config/` and old primitive declarations; no forwarding modules.
- Delete hard-coded protocol constants in moved modules when the profile owns them. If a required constant remains unresolved, fail the phase instead of retaining the old value.
- Remove unused root-package dependencies and duplicate Serde/config parsing paths.

## Security/spec risks

- Bare `u64` aliases permit unit confusion; checked newtypes must distinguish slots, validator indices, and time.
- Arithmetic overflow and hostile profile files can alter timing or bounds.
- Operational configuration must not override consensus profile fields at runtime.
- Moving modules can accidentally keep two crate roots or binaries buildable.

## Positive and negative fixtures

- Positive: zero/min/max permitted primitive values, exact root/domain lengths, and the pinned profile file.
- Negative: overflow/underflow, wrong byte lengths, unknown profile keys, absent required keys, duplicate keys, and attempted runtime consensus override.
- Fixture provenance and hashes live in the phase manifest.

## Interop and differential tests

- Parse primitive/profile vectors from the pinned leanSpec snapshot and compare canonical numeric/byte values.
- Cross-check the same generation in two peer snapshots for diagnostics only.
- Build a test-only JSON report comparing field names and values; production parsing follows the pinned format selected in Phase 00.

## Validation commands

```powershell
cargo metadata --locked --no-deps --format-version 1
cargo check --workspace --all-targets
cargo test -p ethean-primitives -p ethean-profile
cargo clippy --workspace --all-targets --all-features -- -D warnings
rg -n "pub type (Slot|Epoch|Root|BlockHash|ValidatorIndex)|slots_per_epoch|seconds_per_slot" crates
Test-Path src
git diff --check
```

`Test-Path src` must print `False`; search hits must be reviewed profile accesses, not duplicate declarations or literals.

## Exit criteria

- Workspace builds one node binary and has dedicated primitives/profile crates.
- Root `src/` and old config/alias definitions are deleted.
- Every consensus parameter consumed by surviving code comes from the pinned profile.
- Primitive/profile fixtures and negative-boundary tests pass.

## Rollback/data policy

- Revert the complete workspace move; do not keep both layouts.
- Profile serialization is not a production database promise.
- Existing databases remain unsupported prototype data and are not opened by a Lean network identifier.

## Artifacts/evidence

- Workspace metadata graph, phase fixture manifest, boundary-test output, constant search, and migration summary.
- Updated local research note if the pinned profile differs from peer assumptions.

## Dependencies

- Requires phases 00-01.
- Phase 03 uses primitives; phases 04-06 use the profile.
