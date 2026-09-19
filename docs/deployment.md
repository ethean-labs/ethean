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
./target/release/ethean start --ticks 3
./target/release/ethean start --ticks 2 --wall-clock
```

`start` (default) loads `lstar_devnet`, opens Lean in-memory storage, smokes `/lean/v1/health`,
runs a finite **elapsed** duty loop, then drains. `--wall-clock` samples the system clock and
sleeps until the next profile interval between ticks. Full gossip/QUIC swarm bind and RocksDB-backed
production deploy remain open gates (`prepare_transport` / `open_path` fail closed; TCP/WS refused).

## Config / data

- Prefer paths and schema id from `ethean-storage` (`ethean-lc-d5-v1`).
- Refuse directories whose names contain legacy product markers (see storage `refuse_legacy_path`).
- HTTP surface is `/lean/v1/…` via `ethean-rpc` (no `/eth/v1` Beacon compatibility).

## Release tooling

See [release/README.md](./release/README.md) for upgrade/rollback gates and soft legacy scan scripts under `tools/release/`.

## Docker

Container images and compose samples that referenced the retired Beacon binary are withdrawn. Recreate images against `bin/ethean` after QUIC and durable storage gates close.
