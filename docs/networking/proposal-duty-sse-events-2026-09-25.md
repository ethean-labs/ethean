# Proposal duty narrowing + admin events SSE (2026-09-25)

## Why

1. `/lean/v1/validator/duties` listed a `proposal` row for every owned index.
   Lean round-robin already has `proposer_for_slot(slot, n)`.
2. Admin events were drain-only JSON; operators often want a live stream.

## What landed

- Proposal duty rows only when `proposer_for_slot` is in
  `owned_validator_indices` and a proposer key is loaded.
- `GET /lean/v*/events` with `Accept: text/event-stream` keeps a long-lived
  SSE connection (poll JSON still works without that Accept header).
- Pings every 15s; idle close after 60s without data; ends on shutdown.

## Verify

```text
cargo test -p ethean-rpc --lib
curl -s http://127.0.0.1:5052/lean/v1/validator/duties
curl -N -H "Accept: text/event-stream" http://127.0.0.1:5052/lean/v1/events
```
