# Phase 00: Research and Compatibility Snapshot

## Pinned inputs

- Ethean baseline: `880982f9635e8507e5cac37131c3696f6d06191b`.
- leanSpec candidate snapshot: `leanEthereum/leanSpec@0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8`; verify repository identity, commit signature/provenance, and fixture compatibility before accepting it.
- Peer evidence pins: Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, Qlean-mini `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, Gean `b78f6d737f4df57a72d5e230635681235fda8024`, Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.
- Evidence summary: `docs/lean-peer-client-research-library-2026-09-19.md`.

## Objective

Create a reproducible protocol compatibility snapshot that turns moving upstream material into phase-specific immutable inputs. Resolve the protocol generation, schemas, constants, fixtures, cryptography links, and finality rules before production code changes.

## Non-goals

- No Rust implementation, dependency migration, or source-tree cleanup.
- No choice based solely on peer-client consensus.
- No invented defaults for absent or contradictory values.

## Entry criteria

- The baseline commit and current worktree changes are recorded separately.
- `bazalinacaklar/` is writable and remains ignored.
- Upstream repositories and fixture assets can be fetched and hashed.

## Exact affected old and new paths

- Read-only old evidence: `Cargo.toml`, `Cargo.lock`, `src/types/`, `src/crypto/`, `src/consensus/`, `src/network/`, `src/storage/`, `docs/lean-peer-client-research-library-2026-09-19.md`.
- New tracked paths: `spec/pins/README.md`, `spec/pins/phase-00.lock.toml`, `spec/pins/protocol-surface.toml`, `spec/fixtures/phase-00/manifest.toml`, `tests/interop/README.md`.
- New local paths: `bazalinacaklar/lean-spec-snapshot.md`, `bazalinacaklar/protocol-surface-checklist.md`, `bazalinacaklar/compatibility-matrix.md`.

## Ordered implementation tasks

1. Fetch leanSpec at the candidate commit; record canonical remote, commit, tree hash, retrieval date, and license.
2. Identify the target devnet/protocol generation represented by that tree. If the tree mixes generations, record the exact selected subtree and reject ambiguous fixtures.
3. Inventory every normative container, generalized index, list bound, domain, time rule, genesis rule, transition function, fork-choice rule, and finality rule.
4. Resolve `MAX_ATTESTATION_DATA` from the selected schema and fixtures; record the citation and reject the alternate generation.
5. Follow leanSpec dependency references to exact leanSig, leanVM, leanMultisig, or equivalent commits and record parameter-set identifiers. If no exact link exists, block cryptographic phases.
6. Download fixture archives; record URL, release/commit, byte length, SHA-256, internal manifest hashes, and license.
7. Map each old Ethean behavior to `reuse`, `replace`, or `delete`, with source evidence. Mark JSON roots, BLS aggregation, GRANDPA, LMD-GHOST, Beacon epochs/balances, and mock persistence as incompatible unless leanSpec proves otherwise.
8. Record exact Rust toolchain and direct dependency versions selected for implementation; branch dependencies are forbidden.
9. Produce one lock file per planned phase by copying only the inputs that phase consumes.
10. Review all locks against local notes; sign off unresolved items as blockers, not TODO defaults.

## Deletion obligations

- Delete downloaded mutable archives after verified fixtures are vendored.
- Delete any compatibility matrix row that lacks a citation or reproducible command; do not retain guesses as decisions.
- Phase 00 exits with no production source changes and no experimental fallback implementation.

## Security/spec risks

- A wrong repository, mutable branch, stale devnet, or mismatched fixture archive can make all later conformance claims false.
- Cryptographic sizes differ across standalone and leanVM-internal XMSS generations.
- Finality evidence currently spans modified 3SF-mini and roadmap heartbeat/Goldfish directions; only the pinned leanSpec generation is implementable.
- Local Markdown evidence was absent at authoring time, so upstream refresh is mandatory rather than advisory.

## Positive and negative fixtures

- Positive evidence fixture: every vendored file matches both archive and manifest hashes.
- Negative evidence fixtures: altered archive byte, missing license, unknown commit, branch-only dependency, schema/fixture generation mismatch, and duplicate fixture name with different bytes.
- Add a test that fails when any manifest path is unlisted or any digest changes.

## Interop and differential tests

- Run the pinned leanSpec fixture generator/runner and preserve command output plus tool versions.
- Select two peers implementing the same pinned generation and compare schema/root/transition outputs.
- Record peer divergence as evidence; leanSpec output remains expected.

## Validation commands

```powershell
git rev-parse HEAD
git status --short
Get-FileHash spec/fixtures/phase-00/* -Algorithm SHA256
cargo metadata --locked --format-version 1
git diff --check
```

Also run the exact upstream fixture command recorded in `phase-00.lock.toml`; the command cannot be named until the pinned tree is inspected.

## Exit criteria

- Target generation, leanSpec commit/tree, fixture archive digest, cryptography commits/parameters, SSZ schemas, attestation bound, and finality generation are explicit.
- Every later phase lock contains only immutable identifiers.
- No unresolved protocol constant needed by phases 01-06 remains.
- Compatibility matrix and fixture integrity checks are independently reproducible.

## Rollback/data policy

- Revert the complete evidence commit if a pin is invalid; never silently rewrite a lock used by merged implementation.
- A refreshed upstream `main` starts a reviewed lock revision or new migration cycle.
- No production database or key data is read or transformed.

## Artifacts/evidence

- Tracked lock, protocol surface, fixture manifest, and interop instructions at the new paths above.
- Local research notes with source excerpts, URLs, commands, and risk decisions.
- Captured fixture-run output identified by content digest.

## Dependencies

- No implementation-phase dependency.
- Blocks phases 01-06.
