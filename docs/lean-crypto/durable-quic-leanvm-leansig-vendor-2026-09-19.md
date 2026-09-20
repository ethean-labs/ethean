# Durable QuicSwarm + leanVM stub + leanSig vendor patch (2026-09-19)

## QuicSwarm ownership

- `EtheanClient` retains `Option<SwarmFacade>` under `libp2p-quic` after boot.
- Accessors: `swarm()` / `swarm_mut()`.
- `QuicSwarm::pump_once` + `SwarmFacade::pump_quic_once` for gossip/duty loops.
- Gossip topic wiring still open; pump API is ready for the event loop.

## leanVM

- New `crates/crypto/src/backend_leanvm.rs` (pin `e2592df4…`, `LEANVM_FFI_LINKED = false`).
- prove/verify route through the stub; with default `test-aggregate`, tests still pass
  until FFI is linked.
- `FfiStatus.leanvm` is true only when `LEANVM_FFI_LINKED` is true (feature alone is not enough).
- Upstream leanVM is a separate `lean-multisig` workspace (`lean_vm`, `lean_prover`,
  `rec_aggregation`); not linked as a Cargo dep yet (edition 2024 / Plonky3 / sandbox).

## leanSig commit-clean overlay

- Committed: `vendor/leansig/num-bigint-0.5.patch` + README.
- `tools/release/vendor-leansig-bigint-fix.ps1` applies that patch into
  gitignored `bazalinacaklar/leanSig-patched`.
- Full git dep still blocked upstream until leanSig bumps `num-bigint` to 0.5.

## Verification

```text
cargo test -p ethean-crypto --lib
cargo test -p ethean-crypto --features leanvm-backend --lib
cargo test -p ethean-network --features libp2p-quic --lib
cargo check -p ethean-node --features libp2p-quic
```
