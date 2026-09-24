# Checkpoint sync client — 2026-09-24

`ethean start --checkpoint-sync-url <url>` now bootstraps the chain from a
peer's finalized checkpoint, as leanSpec `node/sync/checkpoint_sync.py` and
hive's sync suite expect. Hive passes the full
`http://<helper>:5052/lean/v0/states/finalized` URL; a bare base URL or the
blocks URL are accepted too.

Flow (`crates/node/src/checkpoint_sync.rs`):

1. `GET .../states/finalized` and `GET .../blocks/finalized` with
   `Accept: application/octet-stream`, 60 s timeout each, over a small
   dependency-free HTTP/1.1 client (`checkpoint_http.rs`, plain `http://`).
2. Decode the SSZ `State` and `SignedBlock`; require a non-empty registry,
   `block.slot == state.slot` and `block.state_root == hash_tree_root(state)`.
3. When the anchor is ahead of the local head (or the node only holds
   genesis), replace the head state and root, keep the anchor block as a
   durable blob, clear known payloads, and flush the data directory so a
   restart resumes from the anchor.

A failed fetch logs a warning and the node continues from its local genesis;
it never starts from an unverified pair. The end-to-end test serves a pair
from Ethean's own `/lean/v0` routes and bootstraps a second chain owner from it.

## Live store re-anchoring (same day, after the hive rerun)

hive's checkpoint-sync scenarios (`forkchoice filters nodes before finalized
slot` and friends) expect every visible fork-choice node to sit at or beyond
the finalized checkpoint the client synced from. `apply_anchor` replaced the
head state but left the live `ForkChoiceStore` at its genesis anchor, so
`/lean/v0/fork_choice` kept the slot-0 node and genesis checkpoints.
`apply_anchor` now rebuilds the store from the anchor pair
(`create_store(anchor_state, anchor_block)`), then `sync_from_fork_choice`
copies head and safe target; justified and finalized both equal the anchor.
Test: `anchor_reanchors_the_live_fork_choice_store`.
