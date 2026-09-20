# leanSpec FC runner: wall-clock tick.time + justification (2026-09-20)

## What landed

Fixture `stepType=tick` may carry either:

| Field | Meaning |
| --- | --- |
| `interval` | Absolute store interval counter (already supported) |
| `time` | Unix seconds since epoch → intervals via `(t - genesis) * 1000 / ms_per_interval` |

`ForkChoiceStore` now keeps `genesis_time` and `milliseconds_per_interval`
(from anchor state + profile) so the conversion matches Gean/leanSpec runners.

Past targets (store already ahead) are treated as no-ops so `tickToSlot` on
blocks does not conflict with a later wall-clock tick that resolves to a
slightly earlier interval only when mis-read as a raw counter.

## New coverage

| Fixture | Assert |
| --- | --- |
| `test_block_builder_fixed_point_advances_justification` | 6 imports, 2 ticks, 2 gossip aggregates |
| `test_attestation_target_advances_with_attestations` | 5 body imports |
| `test_valid_gossip_aggregated_attestation` | ≥1 aggregate ingest |
| `test_block_includes_genesis_self_vote` | body attestation import |

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Fixture `checks` / storeSnapshot asserts | Landed for core fields — see `leanspec-fc-checks-snapshot-2026-09-20.md` |
| Cap `tickToSlot` at earliest admissible interval (Gean) | Valid votes use earliest-admissible; blocks keep slot start |
| STF runner / Hive image / A2/A3 / leanVM / bigint | External or separate |
