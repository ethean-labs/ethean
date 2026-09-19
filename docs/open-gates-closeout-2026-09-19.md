# Open gates closeout (2026-09-19)

## Ctrl-C / until-signal

- `RunMode::UntilSignal` + `run_until_signal`
- CLI: `ethean start --until-signal` (wall sleeps until Ctrl-C)

## QUIC / UDP

- `prepare_transport` **binds** a non-blocking UDP listen socket (`BoundTransport`)
- `SwarmFacade::attach_transport` / `dial_quic` — dial still `TransportPending` (no libp2p swarm yet)
- TCP/WS still refused via `reject_non_quic`
- Client boot binds ephemeral port `0` and marks network readiness

## RocksDB

- `RocksEngine` + `Database::open_rocks` behind `ethean-storage/rocksdb`
- `open_path` opens RocksDB when the feature is enabled; otherwise `BackendPending`
- Verified on default (feature off). Feature build needs `libclang` / native toolchain (failed on this Windows host without `LIBCLANG_PATH`)

## leanSig / leanVM FFI

- `ethean_crypto::FfiStatus` / `BackendGap` probe compile-time features
- Production verify/sign remain fail-closed until `leansig-backend` / `leanvm-backend` link successfully (Plonky3 / FFI pins)

## Verification

```text
cargo test -p ethean-node -p ethean-network -p ethean-crypto -p ethean-storage --lib
cargo check -p ethean
```

All passed. `cargo test -p ethean-storage --features rocksdb` needs native RocksDB build deps.

## Remaining external blockers

1. libp2p QUIC-v1 swarm / dial protocol wiring  
2. Host toolchain for `rocksdb` feature (libclang)  
3. leanSig / leanVM crate link + FFI symbols from pinned revisions  
