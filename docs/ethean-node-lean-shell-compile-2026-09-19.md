# ethean-node Lean shell compile (2026-09-19)

## Goal

Unblock `ethean-node` / `ethean` after Phase 13 by removing BLS from the node graph and deleting replaced Beacon node modules.

## Changes

- `crates/node/Cargo.toml`: dropped `blst`, `blstrs`, `bls12_381`, `libp2p`, axum/tower HTTP stack; added `ethean-network`, `ethean-storage`, `ethean-sync`, `ethean-rpc`, `ethean-metrics`.
- Root `Cargo.toml`: removed unused workspace pins for BLS, libp2p, and Beacon HTTP stacks.
- `crates/node/src/lib.rs`: default build exports only Lean modules.
- `crates/node/src/network/`: Lean re-export of `ethean-network`; legacy libp2p sources deleted.
- Deleted legacy trees: `api/`, `bench/`, `config/`, `consensus/`, `integration/`, `optimization/`, `storage/`, `utils/`.
- `crates/node/src/client.rs`: `EtheanClient` uses `ChainOwner`, `ethean_storage::Database`, and `SyncStatus`.
- `bin/ethean/src/main.rs`: simple `tracing_subscriber::fmt::init()`.
- `Cargo.lock`: refreshed after dependency purge.

## Verification

```text
cargo check -p ethean-node -p ethean
```

Succeeded (dev profile). Soft scan: no `blst|blstrs|bls12_381` under `crates/` or `bin/` manifests.

## Still open

- QUIC transport wiring, RocksDB backend, leanSig/leanVM FFI.
- Hard legacy scan gate, fuzz/soak, signed SBOM.
- Full node duty loop (scheduler → owner → gossip/RPC) beyond smoke `start()`.
