# leanSpec fork-choice rejection runner (2026-09-20)

## What landed

`ethean-spec-fixtures` now **executes** mapped fork-choice rejection steps from
filled leanSpec JSON (not only parse/index):

1. Decode `anchorState` / `anchorBlock` / step `block` from leanSpec camelCase JSON
2. `create_store` with `lstar_devnet` + `ForkChoiceOpts::STRUCTURAL`
3. For each step with `valid=false` + mapped `rejectionReason` + `block`, call
   `ForkChoiceStore::on_block` and assert the Ethean `ForkChoiceError`

Verified against the production-scheme vector
`test_block_beyond_future_horizon_rejected` (block at slot 2 while clock at 0 →
`BLOCK_TOO_FAR_IN_FUTURE`).

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
# optional: $env:ETHEAN_LEANSPEC_FIXTURES = "$PWD\.cache\leanspec-fixtures\extracted"
cargo test -p ethean-spec-fixtures --lib runs_beyond
```

Tests auto-pick `.cache/leanspec-fixtures/extracted` when the env var is unset.

## Still open

| Gap | Notes |
| --- | --- |
| Attestation-bearing blocks in fixtures | JSON decode refuses non-empty bodies |
| Tick / vote / valid=true steps | Not run yet |
| Broader rejection map | Expand as vectors need them |
| Hive client Docker/YAML for Ethean | Matrix registration |
| STF fixture runner | Separate from FC |
