# leanSpec FC runner: ticks, imports, more rejections (2026-09-20)

## What landed

Expanded `ethean-spec-fixtures` fork-choice execution beyond a single rejection:

| Step | Behaviour |
| --- | --- |
| `tick` | `on_tick_with(interval, hasProposal)` |
| `block` + `valid=true` | `apply_block_unverified` + `on_block` (empty bodies) |
| `block` + `valid=false` | assert mapped `ForkChoiceError` |

New / confirmed vectors (prod-scheme cache):

- `test_block_beyond_future_horizon_rejected` — rejection only
- `test_block_one_past_horizon_rejected` — tick then `BLOCK_TOO_FAR_IN_FUTURE`
- `test_block_with_fabricated_parent_is_rejected` — import then `UNKNOWN_PARENT_BLOCK`

`UNKNOWN_PARENT_BLOCK` maps to `ForkChoiceError::UnknownParent`.

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Blocks with attestations | JSON decode still refuses non-empty bodies |
| Vote / gossip attestation steps | Not run |
| STF fixture runner | Separate track |
| Hive client image for Ethean | Matrix registration |
