# Phase 05 — Lean state transition

Date: 2026-09-19

## Summary

Added workspace crate `ethean-transition` implementing leanSpec lstar state transition
(pin `0b7d33ec`): `process_slots`, `process_block_header`, attestation-data cap,
3SF-mini justification/finalization, and post-state root check.

Verified block application (`apply_block` / `require_proofs`) rejects empty proofs and
returns `UnsupportedSignature` until XMSS verify (Phase 07/08). Structural path:
`apply_block_unverified` / `state_transition`.

Node consensus modules wrap the crate; Beacon BLS committee/reward/slashing detectors
removed from the transition path. Fork choice and finality gadgets remain Phase 06 stubs.

## Package layout

| Module | Role |
| --- | --- |
| `error.rs` | `TransitionError` mirrored to leanSpec rejection reasons |
| `context.rs` | `TransitionContext { profile }` |
| `outcome.rs` | `TransitionOutcome { post_state, post_state_root }` |
| `slot/process.rs` | `process_slots` |
| `block/validate.rs` | `process_block_header` |
| `block/apply.rs` | `process_block` |
| `operation/attestation.rs` | distinct-data cap + soft structure |
| `operation/justify.rs` | leanSpec `process_attestations` justification body |
| `helpers/*` | proposer round-robin, justifiable distances, justified bits |

## Implemented vs deferred

**Implemented:** slots, header, body attestations + 3SF-mini justify/finalize, state root check, unverified API.

**Deferred:** XMSS/aggregate proof verify; fork-choice store; gossip attestation intake; upstream binary fixture differentials.

## Artifacts

- `spec/pins/phase-05.lock.toml`
- `spec/fixtures/phase-05/`
- Local notes: `bazalinacaklar/phase-05-state-transition.md`
