# Duties interval gating + monitoring SSE note (2026-09-25)

## Why

1. Duties JSON advertised attestation/proposal rows on every tick interval.
   The duty loop only attests on interval 1 and proposes on interval 0.
2. `docs/monitoring.md` still said Lean HTTP on `:5052` was unbound.

## What landed

- Attestation duty rows only when `last_tick.interval == ATTESTATION_INTERVAL`.
- Proposal duty rows only when `last_tick.interval == 0`.
- Monitoring doc lists `:5052` health/duties/events and the SSE curl recipe.

## Verify

```text
cargo check -p ethean-node -q
curl -s http://127.0.0.1:5052/lean/v1/validator/duties
curl -N -H "Accept: text/event-stream" http://127.0.0.1:5052/lean/v0/events
```
