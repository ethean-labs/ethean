# Phase 08 — leanVM aggregation (2026-09-19)

## Summary

Phase 08 adds Type-1 / Type-2 aggregate **statement binding**, fail-closed verify/prove, and a node pool / selection / worker facade.

- `ethean-crypto::aggregation` — `AggregateStatement`, participant order checks, `verify_type1` / `verify_type2`, `prove_*` (leanVM pin `e2592df4…`, `LOG_INV_RATE=2`, max proof 524288).
- `ethean-types::proofs` — bitfield → indices, Type-1 / Type-2 helpers for consensus-derived inputs.
- `ethean-transition::apply_block` — re-derives Type-2 components from the block body, then verifies; empty / mismatched proofs rejected.
- `ethean-node::aggregation` — bounded pool, coverage selection, wall-time budget, sync prover worker (not on chain-owner long-term).

## Backends

| Feature | Status |
| --- | --- |
| `leanvm-backend` | Pin only; FFI not wired → fail closed |
| `test-aggregate` (default) | Statement-digest-bound synthetic proofs for unit tests (not always-true) |

## Tests

- `cargo test -p ethean-crypto aggregation` / full crypto suite
- `cargo test -p ethean-types proofs`
- `cargo test -p ethean-transition`
- Node aggregation unit tests (when `ethean-node` lib test target can compile the module)

## Deferred

1. Native leanVM FFI + dedicated prover process / cgroup
2. Upstream proof fixture byte-diff and 100k mutation corpus
3. Recursive child-merge selection against live gossip (Phase 09/10)
