# leanSpec FC storeSnapshot blockWeights (2026-09-20)

## What landed

### Fork-choice

- `on_block` seeds `latest_known_attestations` from body `aggregation_bits`
  (on-chain aggregates) before `update_head`.
- `insert_known_vote` — LMD newer-vote rule for the known pool.
- `block_weights_from_known()` — ancestor weights from the **finalized** floor
  using relevant known votes (matches fixture dumps).

### Fixture assertions

`storeSnapshot.blockWeights` entries are checked: each listed root’s weight must
match the store computation (`0` if absent).

Aggregated payload pools (`newAggregatedPayloads` / `knownAggregatedPayloads`)
remain deferred.

## Recipe

```powershell
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Payload pool snapshot fields | Need aggregate pool mirroring |
| Hive image / A2/A3 / leanVM / bigint | External |
