# ethean PATH command after cargo build (2026-09-20)

## Goal

Run the client as `ethean …` on Windows and Linux after `cargo build`, without
an `install.sh` / `cargo install` step, and without documenting
`./target/release/ethean` as the primary entry point.

## Change

- Added `bin/ethean/build.rs`. On every build of the `ethean` package it writes
  a shim into `$CARGO_HOME/bin` (default `~/.cargo/bin`):
  - Windows: `ethean.cmd` (removes a stale `ethean.exe` from a prior
    `cargo install` so PATHEXT does not prefer the old binary)
  - Unix: executable `ethean` shell script
- The shim prefers `target/release/ethean`, then `target/debug/ethean`, under
  the workspace root discovered from `CARGO_MANIFEST_DIR`.
- If the shim cannot be written, the build still succeeds and emits a cargo
  warning.

## Docs

- Root `README.md` Quick Start / Install examples use bare `ethean`.
- `docs/deployment.md`, `docs/pq-devnet-5-client-run-2026-09-19.md`,
  `bin/README.md`, and `scripts/README.md` match that workflow.
- Explicitly dropped the install-script path (`install.sh` / `install.ps1`).

## Verify

```bash
cargo build -p ethean
ethean version
```

Requires Cargo's `bin` directory on `PATH` (rustup default).
