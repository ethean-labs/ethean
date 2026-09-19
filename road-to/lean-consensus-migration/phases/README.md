# Lean Consensus Migration Phases

## Pinned inputs

- Ethean planning baseline: commit `880982f9635e8507e5cac37131c3696f6d06191b`.
- Normative upstream snapshots to verify in Phase 00: `leanEthereum/leanSpec` candidates `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54` or `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8`. Neither commit is authoritative until Phase 00 verifies repository identity, fixture compatibility, and records the accepted pin.
- Tracked evidence: `docs/lean-peer-client-research-library-2026-09-19.md`, including its seven peer snapshots.
- Local evidence directory: `bazalinacaklar/` (gitignored). It **does** contain peer library notes and research extracts used during planning; Phase 00 must refresh and extend them before implementation. Local notes supplement upstream pins; they are not protocol authority.
- Every phase consumes its own immutable `spec/pins/phase-XX.lock.toml`; a branch name, moving URL, or unverified constant is not a pin.

## Objective

Replace the Beacon-era `panro` prototype with one leanSpec-derived Ethean production path. Each phase removes the implementation it supersedes before exit. Later phases may depend on untouched legacy areas, but no capability may have both a legacy and Lean production implementation.

## Non-goals

- Peer clients are differential references, not protocol authorities or code templates.
- This sequence does not preserve JSON/Serde consensus encodings, Beacon database records, BLS finality, GRANDPA, or LMD-GHOST compatibility.
- Phases 00–06 do not deliver XMSS signing, recursive aggregation, full networking, production persistence, fleet observability, or release qualification except where a boundary must reject unsupported data.

## Entry criteria

- The worktree is inventoried without modifying or deleting user changes.
- The operator accepts that phase exits are replacement boundaries, not feature flags.
- Phase 00 resolves all constants needed by a later phase from the pinned spec and fixtures; unresolved values **block** that phase.

## Exact old and new paths

- Old roots: `Cargo.toml`, `Cargo.lock`, `src/types/`, `src/config/`, `src/consensus/`, `src/client.rs`, `src/cli.rs`, `src/bin/`, `src/lib.rs`, `src/main.rs`.
- New roots: `crates/primitives/`, `crates/profile/`, `crates/types/`, `crates/ethean-ssz/`, `crates/ethean-genesis/`, `crates/transition/`, `crates/fork-choice/`, `crates/crypto/`, `crates/storage/`, `crates/network-wire/`, `crates/network/`, `crates/sync/`, `crates/validator/`, `crates/node/`, `crates/rpc/`, `crates/metrics/`, `bin/ethean/`, `spec/pins/`, `spec/fixtures/`, `tests/interop/`.
- See [TARGET_WORKSPACE](../03-architecture/TARGET_WORKSPACE.md) for crate layout. Individual phase files below narrow these roots to exact files and deletion sets.

## Phase index

| Phase | Document | Focus |
| --- | --- | --- |
| 00 | [Research and compatibility snapshot](00-research-and-compatibility-snapshot.md) | Protocol lock, fixture pins, open-spec resolution |
| 01 | [Identity and repository cleanup](01-identity-and-repository-cleanup.md) | Ethean naming; delete `panro` identity |
| 02 | [Workspace, primitives, and profile](02-workspace-primitives-and-profile.md) | Virtual workspace, primitives, profile authority |
| 03 | [Canonical SSZ and types](03-canonical-ssz-and-types.md) | Bounded containers, SSZ roots; delete `src/types/` |
| 04 | [Genesis, presets, and clock](04-genesis-presets-and-clock.md) | Deterministic genesis, 4s slot, injectable clock |
| 05 | [State transition](05-state-transition.md) | Pure Lean STF; delete Beacon transition |
| 06 | [Fork choice and finality](06-fork-choice-and-finality.md) | 3SF-mini or pinned Goldfish; delete LMD-GHOST/GRANDPA |
| 07 | [XMSS signer safety](07-xmss-signer-safety.md) | Stateful PQ signing, journal durability |
| 08 | [leanVM aggregation](08-leanvm-aggregation.md) | Type-1/Type-2 proof verify and produce |
| 09 | [Validator and node duties](09-validator-and-node-duties.md) | Proposer, attester, aggregator scheduling |
| 10 | [QUIC, gossip, and req/resp](10-quic-gossip-and-reqresp.md) | Wire protocol, validation, retrieval |
| 11 | [Storage, sync, and checkpoints](11-storage-sync-and-checkpoints.md) | Atomic persistence, sync, checkpoint trust |
| 12 | [API, observability, and multiclient interop](12-api-observability-and-multiclient-interop.md) | Metrics, API, mixed-client interop |
| 13 | [Security, performance, and release](13-security-performance-and-release.md) | Fuzz, chaos, soak, SBOM, legacy deletion verification |

## Critical path

```text
00 → 01 → 02 → 03 → 04 → 05 → 06
         → 07 → 08 → 09
         → 10 → 11
         → 12 → 13
```

1. **Protocol foundation (00–06):** lock spec → identity → workspace → SSZ/types → genesis/clock → state transition → fork choice/finality.
2. **Signer and aggregation (07–09):** XMSS safety → leanVM proofs → validator/node duties.
3. **Network and persistence (10–11):** QUIC/gossip/req-resp → storage/sync/checkpoints.
4. **Interop and release (12–13):** API/observability/multiclient interop → security, performance, and release qualification.

Phases 07–09 require 00–06. Phases 10–11 require 03–09. Phases 12–13 require 03–11.

## Ordered implementation tasks

1. Complete [Phase 00](00-research-and-compatibility-snapshot.md) and commit its evidence lock.
2. Complete [Phase 01](01-identity-and-repository-cleanup.md) and remove `panro` identity.
3. Complete [Phase 02](02-workspace-primitives-and-profile.md) and establish the workspace boundary.
4. Complete [Phase 03](03-canonical-ssz-and-types.md) and delete hand-written Beacon containers.
5. Complete [Phase 04](04-genesis-presets-and-clock.md) and centralize verified network/time inputs.
6. Complete [Phase 05](05-state-transition.md) and delete the Beacon transition pipeline.
7. Complete [Phase 06](06-fork-choice-and-finality.md) and delete legacy fork-choice/finality.
8. Continue through Phases 07–13 in dependency order.
9. At every step, update the phase evidence manifest before declaring exit.

## Deletion obligations

- A replacement phase must delete its listed old files, stale tests, fixture generators, exports, dependencies, and documentation claims.
- Adapters that accept old production data are forbidden. Test-only differential readers must live under `tests/` and cannot be linked by production crates.
- `cargo tree` and repository searches must prove the removed implementation is unreachable and absent.

## Security/spec risks

- Critical blockers include unresolved `MAX_ATTESTATION_DATA`, cryptography parameter generation, exact finality/fork-choice generation (OSD-006), fixture hashes, and checkpoint/restart assumptions.
- JSON roots, unbounded vectors/maps, fake signature paths, BLS-shaped aggregation, and hard-coded Beacon constants must not survive behind aliases.
- Upstream `main` can move. Each phase refreshes deliberately, reviews the diff, and records a new exact commit; automatic floating updates are forbidden.

## Positive and negative fixtures

- Positive suites cover canonical SSZ values, genesis, skipped slots, accepted transitions, justified/finalized progression, and deterministic head selection.
- Negative suites cover malformed lengths/offsets, non-canonical encodings, overflow, wrong roots, future/stale votes, conflicting votes, unknown ancestry, and finality regressions.
- Fixture bytes and expected roots are vendored only with source URL, upstream commit, file hash, and license metadata.

## Interop and differential tests

- Compare Ethean outputs against the pinned leanSpec runner for every normative fixture.
- Use at least two pinned peer snapshots only where they implement the same generation; disagreement is reported, never majority-voted.
- Differential tooling is test-only and must not become a fallback production implementation.

## Validation commands

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo tree --workspace
git diff --check
```

Each phase adds focused commands and repository searches.

## Exit criteria

- All phase-specific tests and the workspace gate pass from a clean checkout.
- No listed old path or forbidden symbol remains.
- The phase lock and evidence manifest identify every normative input by immutable digest.
- There is exactly one production implementation for each migrated responsibility.

## Rollback/data policy

- Roll back only by reverting the complete phase commit; do not reactivate old code with a feature.
- Until a later storage migration is designed, Lean data uses a new schema/network identifier and rejects Beacon-era records.
- Generated test data may be regenerated from pinned sources. Validator keys, signer state, and production databases are never transformed by these phases.

## Artifacts/evidence

- `spec/pins/phase-XX.lock.toml`: immutable source and toolchain revisions.
- `spec/fixtures/phase-XX/manifest.toml`: fixture hashes and provenance.
- `docs/lean-consensus-migration-phase-XX-*.md`: implementation summary produced by each future phase.
- `bazalinacaklar/`: local English research notes and refresh logs; never committed.

## Dependencies

- The strict foundation chain is `00 → 01 → 02 → 03 → 04 → 05 → 06`.
- Phase 03 supplies canonical roots to phases 04–13.
- Phase 04 supplies profile/genesis/clock semantics to phases 05–13.
- Phase 05 supplies validated blocks, states, and operations to Phase 06 and the node pipeline.
- Phase 06 supplies head, safe target, and finality to phases 07–13.
- Phases 07–09 depend on 00–06; phases 10–11 depend on 03–09; phases 12–13 depend on 03–11.
