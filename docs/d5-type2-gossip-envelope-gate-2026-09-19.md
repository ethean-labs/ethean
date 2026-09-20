# D5 Type-2 gossip envelope gate (2026-09-19)

## What landed

Block gossip that carries a `SignedBlock` must include a **single non-empty**
Type-2 proof field (`SignedBlock.proof`). Shape is checked with
`ethean_types::type2_statement_for_block` before verified STF import.

- Bare `Block` payloads still take the structural / smoke path.
- `SignedBlock` with empty or invalid Type-2 envelope is **rejected** (no
  silent fall-through to unverified apply).

## Modules

- `crates/node/src/gossip_type2.rs`
- `crates/node/src/gossip_stf.rs` (import path)

## Still open

- Production leanVM verify of the proof bytes (B2/B3 remain fail-closed).
- Operator fork digest so mesh topics match (C3).
- leanSig production sign/verify for proposer sidecar (B1).
