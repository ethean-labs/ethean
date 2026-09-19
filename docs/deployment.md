# Deployment (Lean Consensus)

Ethean Lean Consensus Client binary: `ethean` (`bin/ethean`).

## Build

```bash
cargo build -p ethean --release
```

Requires the workspace Rust toolchain (see `rust-toolchain.toml`). Do not use legacy Beacon package names or BLS crates.

## Run (smoke)

```bash
./target/release/ethean version
./target/release/ethean start
```

`start` loads the pinned `lstar_devnet` profile and a local smoke genesis, opens the Lean
in-memory store schema, runs a finite duty-tick smoke loop (sync hysteresis + duty gate),
then drains shutdown. Full gossip/QUIC and RocksDB-backed production deploy remain open gates
(`ethean_storage::open_path` fails closed until the RocksDB bind lands).

## Config / data

- Prefer paths and schema id from `ethean-storage` (`ethean-lc-d5-v1`).
- Refuse directories whose names contain legacy product markers (see storage `refuse_legacy_path`).
- HTTP surface is `/lean/v1/…` via `ethean-rpc` (no `/eth/v1` Beacon compatibility).

## Release tooling

See [release/README.md](./release/README.md) for upgrade/rollback gates and soft legacy scan scripts under `tools/release/`.

## Docker

Container images and compose samples that referenced the retired Beacon binary are withdrawn. Recreate images against `bin/ethean` after QUIC and durable storage gates close.
