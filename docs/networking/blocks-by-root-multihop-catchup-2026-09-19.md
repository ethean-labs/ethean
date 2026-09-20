# Multi-hop blocks-by-root parent catch-up (2026-09-19)

## What landed

When a Status-gap fetch returns a tip whose parent is not local head:

1. Buffer the tip as a **sync orphan** (`ChainOwner.sync_orphans`, cap 64).
2. Ask the same peer for missing parent roots (`prepare_blocks_by_root_for_roots`).
3. When a parent imports successfully, **drain** orphans that now match the new head
   (tip applies after mid, etc.).

Modules:

- `ethean-node` `sync_orphan.rs` — orphan cache
- `ethean-node` `blocks_sync.rs` — ingest + drain + `fetch_roots` outcome
- `ethean-network` `blocks_by_root_for_roots` / `prepare_blocks_by_root_for_roots`
- `duty_network` flushes parent follow-up requests after each response

## Why

Single-root Status catch-up only worked when the remote head was exactly one link
ahead. Operator pq-devnet-5 meshes are often many slots ahead of a fresh node.

## Limits / still open

- Orphan map is in-memory only (lost on restart).
- No BlocksByRange yet — deep gaps walk parent-by-parent.
- Crypto gates (leanSig / leanVM Type-2 split) unchanged.
- Operator bootnodes + fork digest still required for a live join.

## Tests

- `orphans_tip_then_applies_after_parent`
- `sync_orphan::take_child_matches_parent`
- `blocks_by_root_for_roots` dedupe / skip zero
