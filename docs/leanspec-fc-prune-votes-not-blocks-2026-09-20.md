# leanSpec FC prune votes not blocks (2026-09-20)

## Goal

Match leanSpec `prune_stale_attestation_data`: on finalization, drop stale
votes/payloads only — **keep every block root** in the store.

## What changed

### Before

`prune_finalized_away` deleted blocks at/below the finalized slot that were not
on the finalized ancestry. That made
`test_prune_finalized_orphaned_branch/*` fail (`blockRoots` got 7, want 8).

### After

- `prune_stale_attestation_data` filters new/known attestation maps and payload
  pools with the leanSpec rule: head slot > finalized **and** descendant of
  finalized
- `on_block` calls that helper when finalized advances
- Durable disk prune (`--data-dir`) is unchanged and separate

### Fixture locks

| Vector | Result |
| --- | --- |
| `test_finalization_prunes_vote_on_orphaned_branch` | green |
| `test_re_gossip_of_pruned_orphaned_vote_is_rejected` | green |

`ethean-spec-fixtures` lib: **42** green.

## Still open

`finalized_safety` (`at_9` empty body vs expected 7-vote payload) remains a
fixture/wire gap, not a block-prune issue.
