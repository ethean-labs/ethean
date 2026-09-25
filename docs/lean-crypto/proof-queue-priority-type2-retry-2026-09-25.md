# Proof queue priority + Type-2 retry window (2026-09-25)

## Problem

Hive same-client 3-validator meshes failed to finalize within 300s while head
still moved (`docs/hive-testing/local-hive-run-2026-09-24.md`). On a shared
host every proposer runs `ethean-prover`, and post-block **Split** jobs shared
one FIFO with critical-path **Block** (Type-2) and **Attestation** (Type-1)
work. A ~3.3s Type-2 merge queued behind Split recovery misses the interval-0
publish window.

## What landed

| Piece | Change |
| --- | --- |
| `proof_service` | Priority inbox: **Block > Attestation > Split** |
| `block_payloads` | Skip Split enqueue while `block_in_flight` |
| `duty_propose` | Mesh Type-2 request allowed on intervals **0..=2** (retry) |
| Duties JSON | Proposal visibility matches intervals 0..=2 |

No synthetic proofs; fail-closed verify path unchanged.

## Smoke

```bash
cargo test -p ethean-node --lib -- proof_service::
cargo test -p ethean-node --lib -- duty_propose_tests
```

## Still open

- Hive sync suite re-run with public (or locally tagged) `:devnet5`.
- Operator GHCR public + ethereum/hive PR + A2/A3 paste.
