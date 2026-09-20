# Owned-index proposer gate (2026-09-20)

## What landed

| Piece | Change |
| --- | --- |
| `duty_propose_gate::is_assigned_proposer` | Empty owned → solo/smoke always; else `slot % n` must be owned |
| `try_plan_proposal` | Early-skip when the node does not own the assigned index |

Hive nodes with `--validator-registry` only propose on their slots. Local smoke
without registry indices keeps the previous always-plan behavior.

## Still open

- Upstream ethereum/hive `clients/ethean`
- Replace interim XMSS-as-proof with leanVM Type-1 for attestation gossip
- Apply `ATTESTATION_COMMITTEE_COUNT` from config.yaml into the live profile
