# Phase 09 — Validator and node duties (2026-09-19)

## Summary

Phase 09 adds deterministic duty scheduling around a single chain-state owner.

### ethean-validator
- `duty_gate` — suppress pre-genesis / syncing / missing parent / profile mismatch / unsafe signer / head lag
- `scheduler` — `(slot, interval, generation)` from profile (4s / 5×800ms)
- `attester` / `proposer` / `aggregator` — gated flows; durable signer for attester/proposer; coverage floor for aggregator

### ethean-node
- `chain_owner` — sole writer; snapshots + stale rejection
- `commands` / `events` / `shutdown` — typed control plane
- `block_builder` — parent selection, transition plan hook, publish deadline

## Tests
`cargo test -p ethean-validator` → **14 passed**

## Deferred
Gossip publish (Phase 10), full transition wiring in builder, multi-node interop, 24h soak.
