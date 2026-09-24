# Live ForkChoiceStore + safe-target attestations (2026-09-24)

## Why

Node `safe_target` previously tracked the justified checkpoint only. Lean
interval-3 safe-target needs a live `ForkChoiceStore`. Without it, local
attesters cannot vote the FC-backed target and metrics stay stubs.

## What landed

- Optional `ChainOwner.fc: Option<ForkChoiceStore>` (structural opts).
- `chain_fc.rs`: genesis `create_store`, `fc_on_block`, `fc_on_tick`,
  `sync_from_fork_choice`, `safe_target_slot` from the store block tree.
- Init after `seal_genesis_head` and after durable head restore (slot-0
  empty-body genesis only; advanced resumes skip init until later work).
- Block apply paths (`local_finality`, gossip STF, proposal proof accept)
  call `fc_on_block` after advancing the linear head.
- Wall duty ticks call `fc_on_tick` before attest/propose.
- `duty_attest` uses `owner.safe_target` for head+target when non-zero.

## Deferred

- FC-owned canonical head (competing tips / LMD replace of linear import).
- Rebuild store from durable tip after non-genesis resume.
- Upstream fixture re-fill for empty-body dumps.

## Verify

```text
cargo test -p ethean-fork-choice --lib
cargo test -p ethean-node --lib -- init_store_sets_safe_target attests_to_safe_target
```

Windows hosts that cannot compile leanVM `system-info` (no `libc::rusage`)
should run the node crate tests under Linux/WSL.
