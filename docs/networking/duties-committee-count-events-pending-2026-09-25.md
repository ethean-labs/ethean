# Duties committee count + events pending (2026-09-25)

## Why

1. Operators verifying `subnet = index % N` needed the profile ACC value on
   the duties response itself.
2. JSON event poll returned only `{events}` with no leftover backlog signal.

## What landed

- `attestation_committee_count` on `/lean/v1/validator/duties`.
- Poll `/lean/v*/events` returns `{events, drained, pending}` after each drain.
- `SharedApiState::events_pending()` for residual backlog depth.

## Verify

```text
cargo test -p ethean-rpc --lib
curl -s http://127.0.0.1:5052/lean/v1/validator/duties
curl -s http://127.0.0.1:5052/lean/v0/events
```
