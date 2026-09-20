# Duty loop and RocksDB gate (2026-09-19)

## Duty loop

- `crates/node/src/dispatch.rs`: applies `ChainCommand` (tick / import / syncing / shutdown) and emits `ChainEvent`.
- `crates/node/src/events.rs`: added `SyncingUpdated(bool)`.
- `crates/node/src/duty_loop.rs`: finite interval-driven loop with sync hysteresis + `evaluate_gate`; ends in shutdown drain.
- `EtheanClient::start` runs `DutyLoopConfig::default()` (5 ticks) after schema verify.

## RocksDB gate

- `StorageError::BackendPending`
- `ethean_storage::open_path`: refuse legacy paths, then fail closed (no silent memory fallback)
- Optional `ethean-storage/rocksdb` feature; bind still pending

## Verification

```text
cargo test -p ethean-node --lib
cargo test -p ethean-storage --lib
```

12 + 12 tests passed.

## Still open

QUIC swarm bind, real RocksDB open, leanSig/leanVM FFI, hard legacy scan, continuous wall-clock duty loop (beyond smoke).
