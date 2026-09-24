# Duties JSON attestation subnet (2026-09-25)

## Why

Attestation rows on `/lean/v1/validator/duties` listed index + slot only.
Operators (and Hive debugging) need the gossip subnet the node will publish on.

## What landed

- `DutyRow.subnet: Option<u16>` — set for attestation rows, omitted for proposals.
- Mapping matches `duty_attest`: `index % attestation_committee_count`.
- JSON includes `"subnet"` only when present.

## Verify

```text
cargo test -p ethean-rpc --lib
curl -s http://127.0.0.1:5052/lean/v1/validator/duties
```
