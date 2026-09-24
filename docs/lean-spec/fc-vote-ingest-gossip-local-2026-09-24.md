# Feed verified votes into the live ForkChoiceStore (2026-09-24)

## Why

Blocks and ticks reached the optional store, but verified gossip / local
attestations never called `on_attestation_data` /
`on_aggregated_attestation`. Pending LMD votes and interval-3 safe-target
updates therefore ignored the network vote path.

## What landed

- `chain_fc_votes.rs`: `fc_on_attestation` / `fc_on_aggregated` (structural).
- `gossip_attestation`: after XMSS / Type-1 verify, feed the store.
- `duty_attest`: feed the store after a successful local sign.
- Votes stay in the pending pool until interval ticks promote them (leanSpec).

## Still external / deferred

- Fixture re-fill for empty-body `at_9` / `dead_9` dumps (upstream).
- Operator A2/A3 digests and bootnodes (paste only; do not invent).
- Hive matrix registration (external).

## Verify

```text
cargo test -p ethean-fork-choice --lib
cargo test -p ethean-node --lib -- vote_lands_in_pending_pool
```
