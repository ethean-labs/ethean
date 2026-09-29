# CI lint gates and node tests (2026-09-29)

## Problem

`.github/workflows/lean-crates.yml` installed rustfmt and clippy but never ran
them, and its test step skipped `ethean-node` and `ethean-spec-fixtures`. The node
crate is where duties, sync, proof collection and the mesh loop live (150+ tests),
so none of that was checked on a PR. Locally the tree had drifted too: about 35
files failed `cargo fmt --check` and `clippy -D warnings` failed across ssz,
crypto, network, rpc, spec-fixtures and node.

## What changed

- CI now runs, in order:
  - `cargo fmt --all --check`
  - `cargo clippy --locked --workspace --all-targets --features ethean-node/libp2p-quic -- -D warnings`
  - the existing Lean crate tests
  - `cargo test --locked -p ethean-node -p ethean-spec-fixtures --features ethean-node/libp2p-quic`
- The workspace is formatted and clippy-clean on toolchain 1.98.1, the same as CI.
  Most fixes were mechanical (`clippy --fix`). The hand-made ones:
  - `api_ssz` / `api_view` / `validator_registry`: merged `if` branches that
    returned the same value.
  - `record_slots` takes a `SlotPanel` struct instead of eight arguments.
  - `Command::Start` boxes `StartArgs` (the enum variant was 424 bytes).
  - `crypto::signature::signature` renamed to `signature::wire` (module
    inception).
  - `spec-fixtures` `HexError` variants lost the `Bad` prefix.
  - Test struct literals use `..Default::default()`.
- `OsRandom` used `/dev/urandom` directly, so XMSS key generation
  (`ETHEAN_PRODUCTION_KEYGEN=1`) failed on Windows. It now uses `getrandom` 0.3,
  which was already in the lock file (`ProcessPrng` on Windows, `getrandom(2)` /
  urandom on Unix). CI runs on Linux, which is why this was never caught.

## Verify

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --features ethean-node/libp2p-quic -- -D warnings
cargo test --locked -p ethean-node -p ethean-spec-fixtures --features ethean-node/libp2p-quic
```
