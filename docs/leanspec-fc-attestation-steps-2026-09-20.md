# leanSpec FC runner: attestation steps (2026-09-20)

## What landed

`ethean-spec-fixtures` now executes leanSpec `stepType=attestation` on the
structural fork-choice path (`on_attestation_data`), including expected
rejections.

| Piece | Role |
| --- | --- |
| `rejection.rs` | Maps gossip/FC attestation reasons (`UNKNOWN_SOURCE_BLOCK`, `ATTESTATION_TOO_FAR_IN_FUTURE`, ancestor/slot mismatches, duplicates, …) |
| `json_types::attestation_from_value` | Decodes validator index + `AttestationData` (signature ignored) |
| `fc_steps::apply_attestation_step` | Optional `tickToSlot`, then import or assert rejection |
| `FcRunReport.attestations` | Counts successful vote ingests |

Confirmed vectors (prod-scheme cache from `fetch-leanspec-fixtures.ps1`):

- `test_attestation_unknown_source_block_rejected` — two imports, one rejection
- `test_attestation_too_far_in_future_rejected` — two imports, one rejection

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Blocks with non-empty attestation bodies | JSON decode still uses empty `BlockBody` |
| `aggregated_attestation` steps | Not wired |
| STF fixture runner | Separate track |
| Hive client image for Ethean | Matrix registration |
| Operator A2/A3 / real leanVM / leanSig bigint | External gates |
