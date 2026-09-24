# Block production follows leanSpec vote selection — 2026-09-24

The proposer used to take the best-coverage pool entry per attestation data,
sorted by message root, and capped the body at 16 entries. leanSpec
`lstar/block_production.py` (pq-devnet-5, PR #897) selects differently, and
the difference showed up as justification lag against other clients.

`crates/node/src/block_builder/spec_select.rs` now implements the spec loop:

- candidates are ordered by `(target.slot, hash_tree_root(data))`;
- a vote is accepted only when its head root is a block this node has seen
  (`ChainOwner::known_block_roots`: head, durable blocks, head history, fork
  choice blocks), its source slot equals the current justified checkpoint,
  its checkpoints lie on the chain view `history ‖ parent ‖ zero per skipped
  slot`, its source slot is justified, and its target is not yet justified;
  genesis self-votes are kept for their head weight;
- the body is capped at `MAX_ATTESTATIONS_DATA` (8, was 16, which peers
  reject);
- after each pass a trial `process_block` runs; when justification or
  finalization moved, the loop re-anchors and scans again;
- a vote the transition refuses is dropped instead of failing the proposal.

One proof is carried per data: the variant covering the most validators, ties
broken by the larger encoding. The spec folds several proofs into a child
merge at this point; Ethean leaves that union to the aggregator round, which
already merges pool variants, so a proposer never waits on an extra Type-1.

Tests: `crates/node/src/block_builder/spec_select_tests.rs` (genesis self-vote
with the widest variant, unknown head, wrong source, off-chain target, empty
pool, chain view) and the existing proposer and local-finality tests.
