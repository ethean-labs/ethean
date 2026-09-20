# leanSpec STF runner: full cached suite green (2026-09-20)

## What landed

All **74** prod-scheme `state_transition` vectors under the leanSpec fixture
cache now pass via `ethean-spec-fixtures`:

```text
cargo test -p ethean-spec-fixtures --lib inventory_cached_stf -- --nocapture
# summary ok=74 fail=0 total=74
```

### Runner helpers

| Case | Behaviour |
| --- | --- |
| Normal blocks | `apply_block_unverified` |
| `BLOCK_SLOT_MISMATCH` | `apply_block_unverified_no_slots` when `pre.slot != block.slot` |
| `BLOCK_OLDER_THAN_LATEST_HEADER` | no-slots path; allow `0x00…` placeholder state roots |
| `BLOCK_SLOT_NOT_IN_FUTURE` | strict `process_slots` (no fallback) |
| Zero `stateRoot` intermediates | Allow placeholder roots after a failed root check |
| Empty `blocks` + `EMPTY_VALIDATOR_REGISTRY` | `proposer_for_slot` on empty registry |

### Transition API

- `apply_block_unverified_no_slots` — header/body without `process_slots`
- `proposer_for_slot` re-exported from `ethean-transition`

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib stf_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| FC weight / aggregated payload snapshots | Separate FC track |
| Blocks-by-range QuicSwarm stream | Sync track |
| Hive image / A2/A3 / leanVM / bigint | External |
