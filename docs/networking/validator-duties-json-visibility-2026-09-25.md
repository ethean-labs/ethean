# Validator duties JSON visibility (2026-09-25)

## Why

`GET /lean/v1/validator/duties` returned an empty `{"duties":[]}` placeholder.
Operators and Hive smoke could not see which local validator indices were
armed for attestation / proposal on the last duty tick.

## What landed

- `DutiesView` / `DutyRow` on the Lean API snapshot.
- Duties are filled from `ChainOwner` on each metrics/API refresh:
  owned indices, attester/proposer loaded flags, aggregator flag, last tick
  slot/interval, and bounded `attestation` / `proposal` rows when not syncing.
- Proposal rows are **visibility candidates** (owned indices), not a full
  round-robin schedule API.

## Still external / later

- Exact proposer schedule / committee assignment wire (when leanSpec schedule
  surface is pinned).
- Long-lived SSE for `/lean/v1/events` (poll already exists).

## Verify

```text
cargo test -p ethean-rpc --lib -- duties_lists_owned_rows
curl -s http://127.0.0.1:5052/lean/v1/validator/duties
```
