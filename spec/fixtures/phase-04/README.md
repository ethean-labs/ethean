# Phase 04 fixtures

Provenance for genesis builder / injectable 4-second slot clock.

- Manifest: [manifest.toml](./manifest.toml)
- Lock: [`../../pins/phase-04.lock.toml`](../../pins/phase-04.lock.toml)
- Crates: `ethean-genesis`, profile extensions (`ForkId`, `profiles/pinned.toml`)

## Coverage (this phase)

Unit tests in `ethean-genesis`:

- Exact 4s slot boundaries and interval indices
- Pre-genesis rejection
- Genesis-time overflow (`checked_mul(1000)`)
- `FakeTime` regression rejection
- `GenesisBuilder` slot 0 + validator count / indices

## Upstream differential (pending)

Phase 00 pins `fixtures-prod-scheme.tar.gz` (sha256 in phase-00 lock). The full archive is not re-vendored here. Until it is available on the authoring host, genesis **state root** comparison against leanSpec fixture vectors remains an open risk.

## Timing under test

| Constant | Value |
| --- | --- |
| `seconds_per_slot` | 4 |
| `intervals_per_slot` | 5 |
| `milliseconds_per_slot` | 4000 |
| `milliseconds_per_interval` | 800 |
| `genesis_time` unit | Unix seconds |
