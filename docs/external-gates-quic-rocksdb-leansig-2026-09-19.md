# External gates progress (2026-09-19)

Follow-up after open-gates closeout: close or narrow the three external blockers.

## libp2p QUIC swarm

- `QuicSwarm` binds QUIC-v1 only (`libp2p` 0.54 + `with_quic`).
- `SwarmFacade::bind_quic_swarm` / `dial_quic_peer` wire the swarm into the facade.
- Feature: `ethean-network/libp2p-quic` (forwarded by `ethean-node/libp2p-quic`).
- Verified: `cargo test -p ethean-network --features libp2p-quic --lib binds_ephemeral_quic`
- UDP `BoundTransport` + `probe_udp_status` remain for path checks without libp2p.

## LLVM / libclang / RocksDB

- `tools/release/check-libclang.ps1` now sets `LIBCLANG_PATH` and `INCLUDE`
  (MSVC Toolset `stdarg.h` + Windows SDK ucrt/shared/um).
- On this host: VS 2017 Community MSVC 14.16 + LLVM bin + WinSDK 10.0.17763.
- Verified: after dot-sourcing the script,
  `cargo check -p ethean-storage --features rocksdb` succeeds.

## leanSig / leanVM

- leanSig pin `c08a3bae…` + `rand 0.10`; wrapper seeds `StdRng` from `rand::rng()`.
- Git dep alone still fails on `num-bigint` 0.4 vs Plonky3 0.5.
- Local vendor: `tools/release/vendor-leansig-bigint-fix.ps1` → path override
  (do not commit) → `cargo check -p ethean-crypto --features leansig-backend` OK.
- `leanvm-backend` remains a compile-time stub; prove/verify fail closed.

## Commands

```text
. .\tools\release\check-libclang.ps1
cargo test -p ethean-storage --features rocksdb --lib
cargo test -p ethean-network --features libp2p-quic --lib
# local only after vendor script + Cargo.toml path override:
cargo check -p ethean-crypto --features leansig-backend
```
