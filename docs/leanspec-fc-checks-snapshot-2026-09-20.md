# leanSpec FC runner: checks + storeSnapshot (2026-09-20)

## What landed

After each executed fork-choice step, the runner validates optional
assertions carried by the fixture:

| Source | Fields covered |
| --- | --- |
| `checks` | `time`, `headSlot`, `headRoot`, justified/finalized slot+root |
| `storeSnapshot` | `time`, `headRoot`, `safeTargetRoot`, justified/finalized checkpoints, `blockRoots` set |

`FcRunReport.assertions` counts steps that carried at least one of these
objects and passed.

Deferred: `blockWeights`, attestation signature pools, aggregated payload
pools, label-based roots, safe-target on-demand recompute beyond store field.

## Vote clock (Gean-aligned)

- Valid attestation / gossip-aggregate steps advance to
  `slot * intervals_per_slot - gossip_disparity` (earliest admissible).
- Invalid (expected rejection) steps **do not** advance time, so
  `ATTESTATION_TOO_FAR_IN_FUTURE` still fires.
- Block `tickToSlot` defaults to **true** when omitted.

## Recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
cargo test -p ethean-spec-fixtures --lib fc_runner
```

## Still open

| Gap | Notes |
| --- | --- |
| Weight / payload snapshot fields | **blockWeights landed** — see `leanspec-fc-block-weights-snapshot-2026-09-20.md`; payload pools still open |
| STF fixture runner | Scaffold landed — see `leanspec-stf-runner-2026-09-20.md` |
| Hive image / A2/A3 / leanVM / bigint | External |
