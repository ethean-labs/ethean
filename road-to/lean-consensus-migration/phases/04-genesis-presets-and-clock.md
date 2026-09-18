# Phase 04: Genesis, Presets, and Clock

## Pinned inputs

- Phase snapshot: `LC-D5-2026-09-19`; Ethean planning baseline `880982f9635e8507e5cac37131c3696f6d06191b`.
- `leanSpec` candidate commits: `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54` or `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8`. Either commit is a **candidate only** until Phase 00 verifies and records the accepted value in `spec/pins/phase-04.lock.toml`.
- Accepted `spec/pins/phase-04.lock.toml` with pinned Lean profile/genesis schemas, genesis fixture bytes/root, **4-second slot** timing, fork/network identity bytes, and allowed clock tolerance if normatively defined.
- No numeric constant is accepted from the current `StateTransitionConfig`, README, Beacon 12-second/32-slot assumptions, or peer client unless the pinned leanSpec snapshot independently defines it.
- Peer evidence (differential only): Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, Qlean-mini `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, Gean `b78f6d737f4df57a72d5e230635681235fda8024`, Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.
- Tracked evidence: `docs/lean-peer-client-research-library-2026-09-19.md`. Local notes under `bazalinacaklar/` supplement Phase 00; unresolved fork-identity or tolerance values are **blockers**.

## Objective

Create one deterministic genesis builder/loader and one injectable Lean slot clock backed by immutable profiles. Remove Beacon 12-second slots, 32-slot epochs, and wall-clock reads from consensus-facing code.

## Non-goals

- No block/state transition, fork-choice weighting, finality, networking scheduling, or validator duty execution.
- No “mainnet” preset unless the pinned spec explicitly defines one.
- No implicit default genesis or zero-state fallback.
- No retention of `slots_per_epoch`, sync-committee timing, or deposit-contract constants from the Beacon prototype.

## Entry criteria

- Phase 03 provides canonical Lean state/container roots.
- Phase 02 profile contains all verified timing/genesis inputs needed by the pinned generation.
- Phase 00 identifies the exact genesis fixture, fork/network identity derivation, and time semantics; unresolved tolerance or overflow behavior blocks entry.

## Exact old and new paths

Replace and then delete Beacon timing/genesis assumptions in:

- `src/consensus/state_transition.rs`, `src/consensus/attestation_processing.rs`, `src/consensus/block_processing.rs`, `src/consensus/performance.rs` (baseline paths; after Phase 02 also under `crates/ethean-node/src/consensus/`).
- `src/client.rs`, `src/api/config.rs`, `src/network/network_config.rs` (and post–Phase 02 equivalents under `crates/ethean-node/src/`).

Create genesis crate:

- `crates/ethean-genesis/Cargo.toml`, `README.md`, `src/lib.rs`, `src/builder.rs`, `src/loader.rs`, `src/clock.rs`, `src/error.rs`.

Extend profile crate per [TARGET_WORKSPACE](../03-architecture/TARGET_WORKSPACE.md):

- `crates/ethean-profile/src/profile.rs`, `src/preset.rs`, `src/fork.rs`, `src/limits.rs`, `src/validation.rs`.
- Committed selected-generation data: `crates/ethean-profile/profiles/pinned.toml`.

Node wiring:

- `crates/ethean-node/src/clock.rs`, `crates/ethean-node/src/client.rs`.

New fixtures/tests:

- `spec/fixtures/phase-04/genesis/`, `spec/fixtures/phase-04/clock/`, `spec/fixtures/phase-04/manifest.toml`, `crates/ethean-genesis/tests/`, `tests/interop/genesis.rs`.

Every hand-written source file is at most **300 lines**; split by responsibility before review.

## Ordered tasks

1. Add the exact selected-generation preset as data, with source fields and digest linked to the phase lock.
2. Validate profile relationships using checked arithmetic and explicit errors; reject zero/overflowing or internally inconsistent values.
3. Implement genesis loading from canonical SSZ and verify its state root and profile/network identity.
4. Implement deterministic genesis construction only for inputs defined by the pinned spec; compare bytes/root with upstream fixtures.
5. Implement a monotonic injectable clock that maps Unix time to slot using the pinned **4-second** slot duration and handles pre-genesis time explicitly.
6. Separate wall-clock acquisition from slot computation; tests use a fake time source.
7. Wire node startup to require an explicit pinned profile and verified genesis; remove `BeaconState::default()` and implicit genesis behavior.
8. Replace direct slot/epoch/timing literals (`12`, `32`, `slots_per_epoch`, `seconds_per_slot: 12`) in surviving code with profile/clock APIs.
9. Encode fork identity bytes exactly as pinned; reject dummy placeholders such as hard-coded `12345678` unless the active devnet profile explicitly requires them.
10. Add boundary tests around genesis time, slot edges, large timestamps, backward wall-clock movement, and arithmetic overflow.

## Deletion obligations

- Delete `StateTransitionConfig` fields/defaults that duplicate profile/genesis/clock authority.
- Delete hard-coded `32`, `12`, `128`, `64`, `256`, stake, reward, epoch, and timeout values when used as protocol constants; unresolved replacements block exit.
- Delete zero/default genesis fallbacks and direct `SystemTime::now()` calls from consensus-facing modules.
- Delete named legacy presets and documentation claims for 12-second slots or 32-slot epochs.
- No adapter that silently converts Beacon timing to Lean timing.

## Security/spec risks

- A one-second/slot boundary error can split proposer, attester, and finality behavior across clients.
- Retaining 12-second or epoch-based APIs while Lean uses 4-second slots causes duty and fork-choice desynchronization.
- Wall-clock rollback must not make the local consensus slot regress.
- Profile substitution or genesis-root mismatch can join the wrong network.
- Overflow in timestamp-to-slot conversion must return an error, not wrap.
- Wrong fork-identity bytes cause silent cross-network gossip before semantic validation fails.

## Positive and negative fixtures

- Positive: pinned genesis bytes/root, exact genesis instant, first slot boundaries at 4-second intervals, skipped-slot calculations, and maximum valid timestamp from upstream vectors if defined.
- Negative: pre-genesis request where forbidden, wrong genesis root, wrong profile/network ID, truncated genesis, timestamp overflow, backward time sample, unknown preset, and modified consensus field.
- Do not invent clock drift tolerances; include such fixtures only when Phase 00 cites the normative rule.

## Interop and differential tests

- Compare built and loaded genesis bytes/root against the pinned leanSpec runner.
- Feed identical timestamps/profile data to Ethean and two same-generation references; compare slots only where their implementations claim the same rule.
- Compare fork-identity bytes with peer status/topic fixtures when available; record peer-only operational drift separately from normative behavior.

## Validation commands

```powershell
cargo test -p ethean-profile -p ethean-genesis --locked
cargo test --test genesis --locked
cargo test --workspace clock --locked
cargo clippy -p ethean-profile -p ethean-genesis --all-targets --locked -- -D warnings
rg -n "SystemTime::now|BeaconState::default|slots_per_epoch|seconds_per_slot:\s*12|slot\s*/\s*32" crates bin
git diff --check
```

Every numeric search hit requires classification in the phase evidence.

## Exit criteria

- Pinned genesis bytes and root match leanSpec.
- All clock boundary/rollback/overflow tests pass deterministically with 4-second slot arithmetic.
- Node startup rejects absent/mismatched profile or genesis.
- No surviving module defines independent protocol timing/genesis constants or Beacon epoch assumptions.
- Every touched source file is at most 300 lines.

## Rollback/data policy

- Revert the whole phase; do not restore implicit defaults behind a flag.
- Genesis/profile identity is part of the new storage namespace; existing prototype data is rejected.
- Changing a pinned preset after use requires a new network/schema identity, not an in-place edit.

## Artifacts/evidence

- Preset source mapping, genesis/clock fixtures and manifest, root comparison, numeric-literal audit, and `docs/lean-consensus-migration-phase-04-genesis-clock.md`.
- Local note under `bazalinacaklar/` records operational clock assumptions not specified normatively.

## Dependencies

- Requires Phases 00–03.
- Phase 05 consumes genesis/profile/clock; Phase 06 consumes profile/clock and genesis anchor; Phases 07–13 consume slot/interval scheduling derived here.
