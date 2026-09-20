# leanSpec STF runner scaffold (2026-09-20)

## What landed

`ethean-spec-fixtures` now runs leanSpec `state_transition_test` JSON:

| Input | Behaviour |
| --- | --- |
| `pre` | Decode `State` |
| `blocks` | Sequential `apply_block_unverified` |
| `rejectionReason` | Expect matching `TransitionError` display prefix |
| `postStateRoot` | Compare `hash_tree_root(post)` |
| `post.slot` | Optional slot pin (full `post` containers are rare) |

Confirmed vectors (prod-scheme cache):

- `test_block_at_large_slot_number` — 1 import + root/slot checks
- `test_zero_length_aggregation_bits_rejects_block` — `EMPTY_AGGREGATION_BITS`

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib stf_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Broader STF suite (~74 vectors) | Expand case-by-case |
| Partial `post` field matrix beyond slot | As fixtures need them |
| Hive client image / A2/A3 / leanVM / bigint | External |
