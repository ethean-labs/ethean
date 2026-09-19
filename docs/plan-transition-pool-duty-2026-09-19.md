# PlanTransition from pool on duty ticks (2026-09-19)

## Builder

- `plan_from_pool` packs `body_from_pool`, runs `process_slots` + `process_block`, sets `state_root`
- Retains the highest-coverage pool proof bytes on `PlanTransition.aggregate_proof` (may be empty)
- `block_root()` exposes the bound signing / tree root

## Chain owner

- `planned_proposal` / `planned_tick` hold the latest local plan for publish-window checks

## Events

- `ChainEvent::ProposalPlanned { root, slot, attestations, publish_allowed }`
- `publish_allowed` is true only on interval 0 within the same-slot publish window

## Duty step

- Gate pass → prune pool → `plan_from_pool` when `head_state` + `profile` exist
- Proposer index is round-robin (`slot % validator_count`)
- Without local state/profile, no proposal event (attestations still retained in the pool)

## Verification

```text
cargo test -p ethean-node --lib block_builder::transition
cargo test -p ethean-node --lib duty_step
cargo test -p ethean-node --lib
```

## Still open

- XMSS/proposer signing into the Type-2 envelope (pool proof is still best-effort bytes)
- leanVM FFI so verified gossip STF can accept non-empty proofs
- leanSig clean git dependency
