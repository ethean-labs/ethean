# Post-block proof recovery and known payloads — 2026-09-24

leanSpec seeds `latest_known_aggregated_payloads` with every vote a block
carries and lets aggregation use those payloads as fallback children
(`fork_choice.py on_block`, `aggregation.py`). The reaggregation vector
`test_post_block_reaggregation` goes one step further: an aggregator splits a
vote's Type-1 proof out of the block proof and merges it with an overlapping
local partial into their union.

Ethean now does both:

- `ChainOwner::known_payloads` records the data root and slot of every vote
  seen on chain, from imported blocks (`gossip_stf`) and own proposals
  (`duty_propose::accept_block_proof`); it is pruned with the pools and feeds
  `lean_latest_known_aggregated_payloads`.
- `block_payloads::seed_known_payloads` queues a `ProofJob::Split` for each
  block vote an aggregator does not already cover at that width. The proof
  service calls `ethean-prover` `split_type2` with the block proof and the
  per-component keys from `block_proof_components`.
- `aggregation_duty::accept_recovered_proof` re-verifies the recovered
  Type-1 in-process, pools it as a variant and marks it known. It is never
  gossiped again; the next `schedule_aggregations` round picks it up as a
  child through the existing greedy coverage selection.

Observer nodes and nodes without a prover only record the roots. The split
runs off the duty thread like every other proof job, so it does not add to
the proposal path.
