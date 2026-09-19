# ethean-node Lean shell compile (2026-09-19)

## Goal

Unblock `ethean-node` / `ethean` after Phase 13 by removing BLS from the node graph and stopping compilation of broken legacy Beacon modules.

## Changes

- `crates/node/Cargo.toml`: dropped `blst`, `blstrs`, `bls12_381`, `libp2p`, axum/tower HTTP stack; added `ethean-network`, `ethean-storage`, `ethean-sync`, `ethean-rpc`, `ethean-metrics`.
- `crates/node/src/lib.rs`: default build exports only Lean modules (`aggregation`, `block_builder`, `chain_owner`, `client`, `clock`, `commands`, `crypto`, `events`, `network`, `shutdown`). Legacy trees (`api`, `consensus`, `storage`, …) stay on disk for the deletion register but are not `mod`’d.
- `crates/node/src/network/mod.rs`: re-exports `ethean-network` only (orphan legacy `.rs` files under `network/` are uncompiled).
- `crates/node/src/client.rs`: `EtheanClient` uses `ChainOwner`, `ethean_storage::Database`, and `SyncStatus`; no Beacon `/eth/v1` API server.
- `crates/node/src/shutdown.rs`: `ShutdownPhase: Default` so `ShutdownState` defaults to `Running`.
- `bin/ethean/src/main.rs`: `tracing_subscriber::fmt::init()` without `env-filter`.
- `Cargo.lock`: refreshed; BLS/libp2p packages leave the node resolution path.

## Verification

```text
cargo check -p ethean-node -p ethean
```

Succeeded (dev profile). Soft scan: no `blst|blstrs|bls12_381` under `crates/node`.

## Still open

- Physical deletion of orphan legacy node sources (`api/`, `consensus/`, old `network/*.rs`, …) per deletion register.
- QUIC transport wiring, RocksDB backend, leanSig/leanVM FFI, hard legacy scan gate, fuzz/soak, signed SBOM.
- Full node duty loop (scheduler → owner → gossip/RPC) beyond smoke `start()`.
