# Proposer binding verify before gossip (2026-09-19)

## Goal

Do not encode or queue proposal gossip until the local proposer signature re-verifies against the same key and block root that were signed.

## Changes

- `ethean-validator` `Signer::verify_duty`: fail-closed re-check (key role, lifetime window, crypto verify).
- `LocalProposer::verify_proposal`: rebuilds the proposal `SigningDuty` and calls `verify_duty`.
- `duty_step::try_plan_proposal`: after `sign_proposal`, requires verify; on failure clears the signature and skips `encode_proposal_gossip`.
- New `ChainEvent::ProposalBindingVerified` emitted only on success.

## Behaviour

| Path | Result |
| --- | --- |
| Sign ok + verify ok | `ProposalSigned`, `ProposalBindingVerified`, gossip ready |
| Sign ok + verify fail | No gossip this tick (fail closed) |
| Sign fail / no proposer | Unsigned plan; structural gossip still allowed |

## Tests

- `signer::tests::verify_duty_roundtrip`
- `local_proposer::tests::smoke_signs_slot_boundary` (includes verify + tamper)
- `duty_step::tests::plans_signs_and_encodes_gossip_when_publish_allowed`

## Follow-ups

- Wire remote gossip ingest to the same verify surface once validator pubkeys are on the local state.
- leanVM FFI still unwired; Type-2 prove remains test-aggregate / stub until vendor link lands.
