# ethean-rpc

Lean HTTP API. Hive interop is `/lean/v0/…`; `/lean/v1/…` is an alias of the same
handlers. **No** `/eth/v1/` Beacon compatibility.

- Public: health, checkpoints/justified, fork_choice, states/finalized (SSZ),
  blocks/finalized (SSZ)
- Admin: aggregator GET/POST, shutdown, event poll or SSE
  (`Accept: text/event-stream` on `/lean/v0/events` / `/lean/v1/events`)
- Operator extras: `/lean/v1/ready`, identity, chain head/finalized/sync,
  `/lean/v1/chain/fork_choice` live-store stats,
  `/lean/v1/validator/duties` owned-index visibility (proposal = round-robin
  `proposer_for_slot` when owned; attestation rows include gossip `subnet` and
  top-level `attestation_committee_count`)
- Body/rate budgets in `limits.rs` (64 MiB bodies for hive test-driver payloads)
