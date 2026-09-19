# Phase 04 — Genesis authority and 4s slot clock (2026-09-19)

## Summary

Added `ethean-genesis` for deterministic Lean genesis construction/loading and an injectable `SlotClock` driven by the pinned lstar profile (4s slots, 5 intervals). Node startup now requires an explicit genesis state; `State::default()` is no longer used as network genesis.

## Package

| Path | Package |
| --- | --- |
| `crates/genesis` | `ethean-genesis` |

Dependencies: `ethean-primitives`, `ethean-profile`, `ethean-types`, `ethean-ssz`, `thiserror`.

### Clock

- `TimeSource` / `FakeTime` / `SystemTimeSource` (node ops only)
- `SlotClock::slot_at_millis`, `interval_at_millis`, `slot_start_millis`
- `GenesisConfig.genesis_time` is Unix **seconds**; convert with `checked_mul(1000)`

### Builder / loader

- `GenesisBuilder` → slot-0 `State` + `hash_tree_root`
- Network genesis requires ≥1 validator; `allow_empty_for_tests` for empty registries
- `load_genesis_ssz` rejects empty payloads and optional root mismatch

## Profile extensions

- `ForkId` — `fork_name()` only; **no** invented 4-byte fork digest (`fork_identifier_bytes` still open from Phase 00)
- `ProfileLimits` / `limits_of`
- `profiles/pinned.toml` citing leanSpec@`0b7d33ec`

## Node wiring

- `crates/node/src/clock.rs` wraps genesis clock
- `EtheanClient::with_genesis` / `from_builder` / smoke `new()` via `local_smoke_genesis`
- API `/config/spec` advertises Lean timing (not Beacon 12s / `slots_per_epoch: 32`)
- Storage no longer falls back to `State::default()` on empty/failed decode
- Discovery defaults no longer ship Ethereum mainnet bootstrap multiaddrs

## Pins / fixtures

- `spec/pins/phase-04.lock.toml` (inherits Phase 00 commit `0b7d33ec…`)
- `spec/fixtures/phase-04/`

## Remaining risks

1. **Upstream genesis fixture root** — prod tarball not vendored; no byte-level leanSpec root differential yet.
2. **`State::ssz_decode`** — full offset decode still deferred; loader rejects empty but cannot load rich SSZ genesis until Phase 05.
3. **`fork_identifier_bytes`** — still unresolved; gossip fork digests not defined.
4. Consensus modules may still carry other Beacon-era literals outside the Phase 04 touch set.
5. **`ethean-node` full compile** — legacy network/storage/API modules still fail `cargo check -p ethean-node` for pre-existing reasons; Phase 04 client/clock/config wiring is in place and `ethean-genesis` / `ethean-profile` tests pass.
6. **Workspace `rocksdb`** — removed invalid `optional = true` from `[workspace.dependencies]` so Cargo 1.98 can parse the manifest (optionality stays on the node feature).
