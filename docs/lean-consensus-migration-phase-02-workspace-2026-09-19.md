# Phase 02 — Workspace, primitives, and profile (2026-09-19)

## Summary

Root `Cargo.toml` is now a **virtual workspace** (no root `[package]`). Legacy `src/` moved into `crates/node`. New crates:

- `ethean-primitives` (`crates/primitives`) — `Hash32`, `Slot`, `Epoch`, `ValidatorIndex`, `Bytes52`
- `ethean-profile` (`crates/profile`) — immutable `ChainProfile` + `lstar_devnet()`
- `ethean-node` (`crates/node`) — library with former root modules
- `ethean` (`bin/ethean`) — thin binary

Root `src/` deleted. Nested `optimization/Cargo.toml` package removed (module files remain unwired).

## Pins / fixtures

- `spec/pins/phase-02.lock.toml` (inherits phase-00/01)
- `spec/fixtures/phase-02/manifest.toml`

## Profile authority

`lstar_devnet()` pins seconds_per_slot=4, intervals=5, XMSS sizes 52/2536, fork_name=`lstar`, etc. Node `Config` / `NodeConfig` stays operational-only; client loads profile at startup. API `/config/spec` and state-transition defaults pull `seconds_per_slot` from the profile.

## Primitive rules

- `Epoch::from_slot(slot, slots_per_epoch)` — no hardcoded 32 inside primitives
- Checked add/sub on `Slot` / `Epoch`; overflow/underflow unit tests
- Transitional `BlockHash = Hash32` alias only in node types

## Validation (when rustc available)

```powershell
cargo metadata --no-deps --format-version 1
cargo check --workspace --all-targets
cargo test -p ethean-primitives -p ethean-profile
Test-Path src   # must be False
```

Toolchain remains TBD in the lock (do not invent `rust-toolchain.toml` version).

## Remaining compile risks

Legacy modules still contain large files (>300 lines) untouched except import/newtype fixes. Full `cargo check` may still fail on:

- Slot/Epoch literal sites missed in non-consensus modules
- `slots_per_epoch` still hardcoded in some consensus helpers (not yet a profile field)
- Heavy optional deps / incomplete APIs in network, optimization (unwired), integration
- Pre-existing broken symbols in legacy paths

Phase 03+ should continue replacing Beacon containers and splitting oversized modules.
