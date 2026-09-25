# Development

Workspace layout and day-to-day engineering commands.

Short overview: [root README — Development](../../README.md#development).

## Project structure

- `crates/` — library crates (`primitives` … `metrics`; see
  [`crates/README.md`](../../crates/README.md))
- `bin/ethean` — node binary
- `spec/` — phase locks and fixtures
- `docs/` — design notes; operator guides under `docs/readme/`
- `artifacts/`, `scripts/`, `tools/` — evidence, helpers, utilities
- `tests/{interop,security,recovery}/` — cross-crate suites
- `road-to/` — migration planning library

Legacy Beacon-shaped modules under `crates/node` are being replaced phase by phase.

## Building

```bash
cargo build -p ethean
cargo build -p ethean --release
cargo doc -p ethean --open
```

## Day-to-day tools

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test
cargo audit
```

## Adding features

1. Pick the crate that owns the concern (types, transition, network, node, …).
2. Keep each source file at 2000 lines or fewer; split by responsibility.
3. Add tests with the change; keep public docs in English.
4. Follow [CONTRIBUTING.md](../../CONTRIBUTING.md) and
   [source-file-size-limit.md](../source-file-size-limit.md).

## Code quality

- Clippy + rustfmt on the workspace
- LeanSpec fixture runners under `crates/spec-fixtures`
- English-only identifiers, comments, and commit messages
- No AI / Cursor attribution in git metadata
