# CONTRIBUTING.md guide (2026-09-20)

Added a root [CONTRIBUTING.md](../CONTRIBUTING.md) as the single contributor entry
point for Ethean Lean Consensus Client.

## Contents

- Prerequisites: `rust-toolchain.toml` (1.98.1), optional Docker for Grafana
- Setup: `cargo build` / `test` / `clippy` / `fmt`
- Operational default: `pq-devnet-4`; D5 remains a ready path
- Metrics: scrape `:9100` by default; `--metrics` starts Docker Compose for
  Grafana `:3000` and Prometheus `:9090`
- House rules: English-only, 300-line source files, no AI git attribution,
  `.githooks` via `core.hooksPath`
- Peers (Ream, Zeam, ethlambda, …) as interop references only
- Pointers into `docs/` for deployment, path shim, versioning, and peer list

## README cleanup

Duplicate inline Contributing sections in the root README were replaced with a
short link to `CONTRIBUTING.md`. A misplaced Contributing block that sat above
API response examples was removed so that section reads as API docs again.

## Index

Linked from [docs/README.md](./README.md) and the root README table of contents.
