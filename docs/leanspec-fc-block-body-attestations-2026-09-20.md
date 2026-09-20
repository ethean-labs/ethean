# leanSpec FC runner: block body attestations (2026-09-20)

## What landed

Fixture `Block` JSON decode no longer refuses non-empty bodies.
`aggregationBits.data` + `AttestationData` become `AggregatedAttestation`
rows inside `BlockBody`, then `apply_block_unverified` runs the usual
structural STF path.

| Piece | Role |
| --- | --- |
| `aggregated_attestation_from_value` | Body / aggregate payload decode |
| `block_from_value` | Builds `BlockBody::new(...)` instead of empty default |

Confirmed vector:

- `test_block_with_maximum_attestations` — 9 imports (final body has 8 aggregates)

About 50 other fork-choice fixtures in the cache also carry body attestations;
they are now decodable. Add targeted runners as they are needed for pq-devnet
interop confidence.

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| `aggregated_attestation` gossip steps | Landed as `gossipAggregatedAttestation` — see `leanspec-fc-gossip-aggregated-2026-09-20.md` |
| Broader FC suite (finality / reorg / LMD) | Partial — [leanspec-fc-finality-reorg-lmd-2026-09-20.md](./leanspec-fc-finality-reorg-lmd-2026-09-20.md) |
| STF fixture runner | Separate track |
| Hive client image / A2/A3 / leanVM / bigint | External gates |
