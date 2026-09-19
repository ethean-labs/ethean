# Open gates closeout (2026-09-19)

## Ctrl-C / until-signal

- `RunMode::UntilSignal` + `run_until_signal`
- CLI: `ethean start --until-signal` (wall sleeps until Ctrl-C)

## QUIC / UDP

- `prepare_transport` **binds** a non-blocking UDP listen socket (`BoundTransport`)
- `QuicSwarm` + `SwarmFacade::bind_quic_swarm` / `dial_quic_peer` under `libp2p-quic`
- TCP/WS still refused via `reject_non_quic`
- Client boot binds ephemeral port `0` and marks network readiness
- See also [external-gates-quic-rocksdb-leansig-2026-09-19.md](./external-gates-quic-rocksdb-leansig-2026-09-19.md)

## RocksDB

- `RocksEngine` + `Database::open_rocks` behind `ethean-storage/rocksdb`
- `open_path` opens RocksDB when the feature is enabled; otherwise `BackendPending`
- Windows: dot-source `tools/release/check-libclang.ps1` (LIBCLANG_PATH + MSVC INCLUDE)

## leanSig / leanVM FFI

- `ethean_crypto::FfiStatus` / `BackendGap` probe compile-time features
- leanSig: local vendor path after `vendor-leansig-bigint-fix.ps1` compiles `leansig-backend`
- leanVM: still fail-closed stub (`leanvm-backend` has no FFI symbols)

## Verification

```text
cargo test -p ethean-node -p ethean-network -p ethean-crypto -p ethean-storage --lib
cargo check -p ethean
. .\tools\release\check-libclang.ps1
cargo test -p ethean-storage --features rocksdb --lib
cargo test -p ethean-network --features libp2p-quic --lib
```

## Remaining external blockers

1. Commit-clean leanSig git dep (upstream `num-bigint` vs Plonky3) — vendor works locally  
2. leanVM FFI crate / symbols from pin  
3. Node boot optionally attaching `QuicSwarm` (API ready; default still UDP facade)
