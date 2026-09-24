# Genesis body root and integer isqrt — 2026-09-24

Two silent divergences from leanSpec `lstar` were fixed.

**Genesis header body root.** `GenesisBuilder` filled `latest_block_header.body_root`
with zero. leanSpec `generate_genesis` uses `hash_tree_root(BlockBody(attestations=[]))`,
which with the 4096-entry list limit is
`0xdba9671bac9513c9482f1416a53aabd2c6ce90d5a5f865ce5a55c775325c9136`. The zero root
changed the genesis state root, the genesis header root, the slot-1 `parent_root`,
`historical_block_hashes[0]` and the slot-0 justified and finalized roots, so a node
built from `config.yaml` could never agree with a peer on any block root. The
constant is pinned as `EMPTY_BLOCK_BODY_ROOT` in `crates/genesis/src/builder.rs` and
a test recomputes it from `BlockBody::default()`.

**Integer square root.** `is_justifiable_after` used `f64::sqrt`; leanSpec uses
`math.isqrt`. The helper is now an exact integer floor square root and is tested
against a reference loop for deltas up to 10 000 and near `u64::MAX`.

Still open: pin the full genesis state root against a leanSpec-generated
`genesis.ssz` for a small validator set (needs the leanSpec CLI run locally).
