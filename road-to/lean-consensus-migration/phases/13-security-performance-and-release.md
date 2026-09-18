# Phase 13 — Security, Performance, and Release

## Pinned inputs

- Snapshot `LC-D5-2026-09-19`; Ethean baseline `880982f`; full protocol/crypto pins from Phases 00–08; observability pins from Phase 12.
- Security gates: [../04-risks/SECURITY_GATES.md](../04-risks/SECURITY_GATES.md) and performance budgets: [../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md).
- Retirement register: [../05-retirement/DELETION_REGISTER.md](../05-retirement/DELETION_REGISTER.md) and allowlisted legacy-name policy in [../05-retirement/REQUIRED_DIRECTORY_POLICY.md](../05-retirement/REQUIRED_DIRECTORY_POLICY.md).
- Rust `1.97.1`; lockfile, builder container digest, target triple, SBOM generator version, and release signing policy are frozen in `spec/pins/phase-13.lock.toml`.
- Promotion sequence: offline vectors → single-node no-signing → isolated signing → homogeneous multi-node → mixed-client multi-node → adversarial staging → targeted devnet.

## Objective

Qualify Ethean for release through fuzz, chaos, and soak campaigns, reproducible artifact builds with SBOM, verified upgrade/rollback paths, and final legacy deletion verification so no Panro/Beacon production path remains outside the explicit allowlist.

## Non-goals

- No waiver of G1–G2 gates (crypto isolation, XMSS state safety) or checkpoint trust boundaries.
- No release from a dirty worktree, floating dependency pin, or failed deletion verification search.
- No claim of mainnet readiness beyond the pinned devnet/profile scope.
- No permanent retention of legacy names in user-facing strings except documented allowlist entries.

## Entry criteria

- Phases 00–12 pass; homogeneous and mixed-client observability gates are green.
- All prior phase deletion obligations are recorded; open rows in the deletion register have named owners or block release.
- Fuzz corpora, chaos scenarios, soak duration, and performance envelopes are approved from Phase 00 evidence.
- Upgrade/rollback runbooks, migration binaries, and release artifact layout are drafted.

## Exact old and new paths

Replace then delete remaining legacy surfaces:

- Residual `src/` tree, `panro` package names, Beacon-era benches/examples, and documentation that presents legacy APIs as current.
- Release scripts or CI jobs that build floating pins or skip SBOM/deletion checks.
- Any production `cfg` path still reachable to mock crypto, fake prover, TCP/Beacon networking, JSON storage, or Beacon HTTP routes.

Create:

- `tools/release/{repro-build.sh,repro-build.ps1,sbom.sh,artifact-manifest.toml}`.
- `tests/fuzz/` workspace harnesses for SSZ/Snappy, network ingress, RPC parsing, and storage decode.
- `tests/chaos/{partition.rs,resource_exhaustion.rs,process_kill.rs}` and `tests/soak/{multinode.rs,prover_pressure.rs}`.
- `tests/retirement/legacy_name_scan.rs` and `tests/release/reproducibility.rs`.
- `docs/release/{upgrade.md,rollback.md,gates.md}` and `artifacts/phase-13/` evidence layout.

New directories require English `README.md` files; all hand-written source files are at most 300 lines.

## Ordered tasks

1. Run structured fuzz campaigns on untrusted ingress: SSZ/Snappy decode, RPC bodies, network frames, and storage records. Seed from Phase 10/11 negative fixtures; track coverage and crashes.
2. Execute chaos matrix: peer partition, clock skew, disk full, read-only data dir, prover hang/kill, signer crash boundaries, exporter overload, and checkpoint install interruption.
3. Run multi-node soak at approved duration with duty load, mixed-client peers where available, and leak checks on CPU, memory, FDs, stream tables, and metric cardinality.
4. Measure four-second critical path per G6: slot-phase histograms, worst accepted percentiles, and declared safety reserve against [PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md).
5. Produce SBOM and license/vulnerability report for release artifacts; fail on policy violations or unexplained transitive changes.
6. Implement reproducible builds: two clean builders from pinned source, toolchain, lockfile, and container digest must yield matching normalized artifacts and manifest hashes.
7. Validate upgrade path: stage new schema/binary, atomic activation, readiness transition, and post-upgrade interop smoke on homogeneous and mixed-client topologies.
8. Validate rollback path: retain prior generation, prove read-only or resync behavior when downgrade is unsupported, and document signer journal immutability across rollback attempts.
9. Run final legacy deletion verification searches from [DELETION_REGISTER.md](../05-retirement/DELETION_REGISTER.md); close every row or block release with explicit exception record (none permitted for signing/checkpoint gates).
10. Scan repository for forbidden symbols (`panro`, `Panro`, `Beacon`, `/eth2/`, `blst`, mock gossip, JSON consensus storage, empty `NetworkManager`) outside the allowlist in retirement policy.
11. Package release artifacts with provenance: source commit, lock hash, toolchain, builder digest, features, protocol pins, SBOM hash, test evidence summaries, and signatures.
12. Execute promotion sequence to targeted devnet; any failure returns to the earliest affected gate.

## Deletion obligations

- Remove the last reachable legacy production paths listed in Phases 01–12 and the deletion register, including obsolete examples, benches, and docs that imply Beacon compatibility.
- Delete release features or environment switches that re-enable legacy stacks.
- Allowlist-only legacy names: migration records under `road-to/`, historical git tags, and explicitly classified filenames in `road-to/` may mention Panro/Beacon; nowhere else in `crates/`, `bin/`, `tests/`, `examples/`, `benches/`, `docs/`, `deploy/`, or root manifests.
- Final verification searches must be empty or allowlist-documented before exit.

## Security/spec risks

- Fuzz gaps on req/resp framing or checkpoint bundles can hide remote crash or resource exhaustion bugs.
- Soak tests that omit mixed-client peers miss cross-client finality divergence.
- Reproducibility drift from unstated environment variables or timestamps breaks supply-chain guarantees.
- Incomplete deletion leaves a latent Beacon or mock-crypto path reachable through tests linked into production features.
- Upgrade without retained prior generation can brick nodes; rollback without signer policy can cause leaf reuse if attempted on signer data.

## Positive and negative fixtures

- Positive: release manifest golden files, SBOM snapshots, reproducibility hash records, passing gate summaries, and promotion-sequence smoke inputs.
- Negative: builds with forbidden feature combinations, legacy name hits outside allowlist, reproducibility mismatch, failed chaos injection without alert, and upgrade/rollback scenarios that leave mixed schema state.
- Fixture provenance and hashes live in `spec/fixtures/phase-13/manifest.toml`.

## Interop and differential tests

- Mixed-client devnet promotion with at least two Ethean nodes and one pinned peer through finality under fault injection.
- Differential replay of critical vectors against leanSpec runner and available peer fixtures after the full stack is assembled.
- Post-upgrade and post-rollback interop smokes on the same profile to prove data and protocol continuity or fail-closed behavior.

## Validation commands

```text
cargo +1.97.1 test --workspace --locked
cargo +1.97.1 test --test legacy_name_scan --locked
cargo +1.97.1 test --test reproducibility --locked
cargo +1.97.1 test --test partition --test resource_exhaustion --release --locked -- --ignored
cargo +1.97.1 test --test multinode --test prover_pressure --release --locked -- --ignored
cargo fuzz run ssz_decode -- -max_total_time=3600
cargo deny check
tools/release/sbom.sh
tools/release/repro-build.sh --verify
rg -n "panro|Panro|Beacon|/eth2/|blst|NetworkManager|mock.*gossip" crates bin tests examples benches docs deploy --glob '!road-to/**'
rg -n "panro|Panro" Cargo.toml Cargo.lock crates bin tests examples benches docs deploy
```

Allowlist hits require an explicit entry in `road-to/lean-consensus-migration/05-retirement/DELETION_REGISTER.md` or release is blocked.

## Exit criteria

- G0–G8 security gates in [SECURITY_GATES.md](../04-risks/SECURITY_GATES.md) pass with archived evidence; no waivers on signing, checkpoint, or crypto isolation gates.
- Fuzz, chaos, and soak campaigns complete within approved budgets without unresolved crashes or leaks.
- SBOM and reproducibility checks match across two clean builders.
- Upgrade and rollback runbooks are exercised; unsupported rollback fails closed with actionable errors.
- Final legacy deletion verification is clean outside the documented allowlist; no Panro/Beacon production names remain in active code paths.
- Targeted devnet promotion succeeds; every touched source file is within 300 lines.

## Rollback/data policy

Release rollback uses retained artifact generation and prior database/checkpoint generation only when schema support is explicit; otherwise require trusted checkpoint resync. Signer journals and completed duty records are never rolled back by the release tooling. Failed release publication does not mutate deployed signer state.

## Artifacts/evidence

- Gate reports G0–G8, fuzz/coverage summary, chaos matrix results, soak leak report, performance envelope proof, SBOM, reproducibility attestation, upgrade/rollback exercise logs, deletion verification output, allowlist audit, devnet promotion record, and signed release manifest in `artifacts/phase-13/`.

## Dependencies

Depends on Phases 00–12. This phase closes the migration program for the pinned profile; later profile bumps reopen Phase 00 refresh and targeted delta phases only.
