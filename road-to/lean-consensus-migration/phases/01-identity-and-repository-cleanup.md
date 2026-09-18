# Phase 01: Identity and Repository Cleanup

## Pinned inputs

- `spec/pins/phase-01.lock.toml`, produced and accepted in Phase 00.
- Ethean baseline identity findings at `880982f9635e8507e5cac37131c3696f6d06191b`.
- Product identity: `Ethean Lean Consensus Client`; crate and binary identifier: `ethean`.

## Objective

Remove the historical `panro`/generic Beacon identity and false production claims before protocol replacement begins. Keep behavior unchanged except for names, metadata, and deletion of demonstrably nonexistent commands/features.

## Non-goals

- No SSZ, type, genesis, transition, fork-choice, finality, cryptography, networking, or storage implementation.
- No compatibility `panro` binary or crate alias.
- No claim that compilation demonstrates Lean conformance.

## Entry criteria

- Phase 00 accepts the target protocol generation and Phase 01 lock.
- Current user moves/deletions are reconciled without overwriting unrelated work.
- A repository-wide identity and command inventory is attached to the phase evidence.

## Exact affected old and new paths

- In-place old-to-new: `Cargo.toml` package/bin `panro` -> `ethean`; `Cargo.lock` package entry; `src/lib.rs`; `src/client.rs`; `src/cli.rs`; `src/bin/main.rs`; `src/bin/benchmark.rs`; `src/main.rs`.
- Rename symbols in place: `PanroClient` -> `EtheanClient`; `PANRO`/`Panro`/`panro` user-facing strings -> Ethean equivalents.
- Documentation cleanup in place: `README.md`, `src/README.md`, `src/bin/README.md`, and affected module `README.md` files.
- New evidence: `spec/fixtures/phase-01/manifest.toml`, `tests/interop/identity.rs`, `docs/lean-consensus-migration-phase-01-identity.md`.

## Ordered implementation tasks

1. Enumerate every case-insensitive `panro`, `Beacon Chain`, `Beam Chain`, unsupported command, benchmark, endpoint, and performance claim.
2. Rename package, default binary, executable target, imports, public client type, CLI metadata, logs, and test expectations to Ethean.
3. Remove the old binary target entirely; do not ship a shim or symlink.
4. Rewrite root and folder documentation to describe only currently executable behavior and clearly label pre-migration protocol code as non-conformant.
5. Remove commands, features, metrics, endpoints, test counts, and benchmark numbers that cannot be demonstrated by repository code and validation output.
6. Update examples/tests that compile against the renamed crate; delete examples that document nonexistent APIs.
7. Add identity tests that inspect package metadata, `--help`, `--version`, and built artifact names.
8. Record repository searches and command output in the phase summary.

## Deletion obligations

- Delete all `panro` package, binary, symbol, log, path, and public-string compatibility.
- Delete stale identity-only files after content is moved; do not keep duplicate manifesto/architecture files at root and `road-to/`.
- Delete unsupported README command blocks and fabricated performance/coverage claims.
- Delete any example that cannot compile against the one Ethean binary.

## Security/spec risks

- A hidden legacy binary can execute obsolete consensus rules after later phases.
- Misleading “production-ready,” “complete,” or “build passing” claims can cause unsafe deployment.
- Package renaming can leave lockfile, service scripts, or integration imports pointing at a stale executable.

## Positive and negative fixtures

- Positive: Cargo metadata names only `ethean`; CLI help and version identify Ethean; one node binary is emitted.
- Negative: launch/import attempts using `panro` fail; docs test rejects forbidden legacy names and unsupported commands.
- Fixture manifest records expected CLI stdout patterns without embedding unstable build paths.

## Interop and differential tests

- No protocol differential test is valid in this phase.
- Compare `cargo metadata` and CLI surface before/after to prove behavior changed only at the identity boundary.
- Preserve a test-only baseline report; do not compile or execute an old binary.

## Validation commands

```powershell
cargo metadata --locked --format-version 1
cargo build --locked --bins
cargo test --locked --test identity
cargo run --locked --bin ethean -- --help
rg -n -i "panro|grandpa|production-ready|complete coverage|build-passing" Cargo.toml src README.md
git diff --check
```

The legacy-name search must return no production identity; historical planning references may remain only when explicitly labeled historical.

## Exit criteria

- Package, binary, CLI, logs, imports, and current docs use Ethean exclusively.
- Exactly one node executable exists and `panro` invocation is unsupported.
- Current documentation contains no unverifiable runtime or conformance claim.
- All listed obsolete duplicates/examples are removed or explicitly excluded by preserved user work.

## Rollback/data policy

- Roll back by reverting the whole phase.
- Binary/service operators must update executable names atomically; there is no dual-name transition.
- No database format changes occur.

## Artifacts/evidence

- Identity fixture manifest, CLI test, repository search output, Cargo metadata output, and phase summary.
- Phase summary records any historical document intentionally retained and why it cannot be mistaken for current behavior.

## Dependencies

- Requires Phase 00.
- Phase 02 consumes the final `ethean` package and binary names.
