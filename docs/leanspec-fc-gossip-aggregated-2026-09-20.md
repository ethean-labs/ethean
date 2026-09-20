# leanSpec FC runner: gossipAggregatedAttestation (2026-09-20)

## What landed

Structural path for leanSpec `stepType=gossipAggregatedAttestation`:

| Piece | Role |
| --- | --- |
| `ForkChoiceError::EmptyAggregationBits` | Empty participation bitfield |
| `ForkChoiceStore::on_aggregated_attestation` | Validate once; pending-vote each set bit |
| `signed_aggregated_from_value` | `data` + `proof.participants` (proof bytes ignored) |
| `apply_gossip_aggregated_step` | Import / expected rejection |

Confirmed vectors:

- `test_aggregated_attestation_head_slot_mismatch_rejected` → `HEAD_SLOT_MISMATCH`
- `test_gossip_aggregated_attestation_empty_participants_rejected` → `EMPTY_AGGREGATION_BITS`

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Broader gossip-aggregate suite (disparity / valid accept) | Expand case-by-case |
| Finality / reorg / LMD FC suites | Decode + steps mostly ready |
| STF runner / Hive image / A2/A3 / leanVM / bigint | External or separate tracks |
