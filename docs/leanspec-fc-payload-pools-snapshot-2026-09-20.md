# leanSpec FC storeSnapshot aggregated payloads (2026-09-20)

## What landed

Fork-choice now mirrors leanSpec `latest_new_aggregated_payloads` /
`latest_known_aggregated_payloads`:

| Event | Pool |
| --- | --- |
| Block body aggregates | `latest_known_payloads` |
| `gossipAggregatedAttestation` | `latest_new_payloads` |
| `accept_new_attestations` (tick promote) | new → known (merge participant sets) |

Fixture `storeSnapshot.knownAggregatedPayloads` /
`newAggregatedPayloads` entries are asserted as a **coverage** check: every
listed `dataRoot` + `participantSets` must match; the store may retain extra
historical payloads.

## Recipe

```powershell
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Exact payload-set equality / prune policy | leanSpec may drop stale roots more aggressively |
| Hive client Docker/YAML | Matrix registration |
| A2/A3 / leanVM / bigint | External |
