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
./target/release/ethean start --until-signal
./target/release/ethean start --data-dir ./ethean-data
./target/release/ethean validator
```

`start` (default) loads `lstar_devnet`, opens Lean in-memory storage, binds an ephemeral UDP
listen for the QUIC facade, smokes `/lean/v1/health`, runs a finite **elapsed** duty loop, then
drains. `--wall-clock` samples the system clock between ticks. `--until-signal` runs until Ctrl-C.
`--data-dir` uses `ethean_storage::open_path` (requires a build with `ethean-storage/rocksdb`).

UDP Status path probes are available via `probe_udp_status` (reachability only).
Enable `ethean-network/libp2p-quic` (or `ethean-node/libp2p-quic`) for `QuicSwarm` /
`SwarmFacade::bind_quic_swarm` + `dial_quic_peer`. leanSig links via a local vendor
patch (`tools/release/vendor-leansig-bigint-fix.ps1`); leanVM FFI is still fail-closed.
For RocksDB on Windows, dot-source `tools/release/check-libclang.ps1` (sets
`LIBCLANG_PATH` and `INCLUDE` for MSVC/WinSDK), then build with `ethean-storage/rocksdb`.

## Config / data

- Prefer paths and schema id from `ethean-storage` (`ethean-lc-d5-v1`).
- Refuse directories whose names contain legacy product markers (see storage `refuse_legacy_path`).
- HTTP surface is `/lean/v1/…` via `ethean-rpc` (no `/eth/v1` Beacon compatibility).

## Release tooling

See [release/README.md](./release/README.md) for upgrade/rollback gates and soft legacy scan scripts under `tools/release/`.

## Docker

Container images and compose samples that referenced the retired Beacon binary are withdrawn. Recreate images against `bin/ethean` after QUIC and durable storage gates close.
