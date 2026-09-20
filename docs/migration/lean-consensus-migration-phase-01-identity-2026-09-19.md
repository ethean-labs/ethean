# Phase 01 — Identity and repository cleanup

Date: 2026-09-19  
Branch: `plan/lean-consensus-migration`  
Lock: [`spec/pins/phase-01.lock.toml`](../../spec/pins/phase-01.lock.toml)

## What changed

- Root package/binary renamed from `panro` to `ethean` (`Cargo.toml`, `Cargo.lock`).
- `PanroClient` → `EtheanClient`.
- CLI, logs, user-agent, API service strings, and optimization crate metadata now say Ethean.
- Default binary name is `ethean`.
- Identity smoke tests: `tests/interop/identity.rs`.

## Explicitly unchanged (later phases)

- Consensus types still named `BeaconBlock` / `BeaconState` until Phase 03.
- BLS / local WOTS / JSON roots remain until crypto and types phases.
- No SSZ, networking, or storage replacement in this phase.

## Validation

```powershell
rg -n -i '\bpanro\b' src Cargo.toml --glob '!road-to/**'
cargo test --test identity
```

`rustc` was unavailable on the authoring host; compile verification is required on a toolchain-equipped machine before Phase 01 exit sign-off.
