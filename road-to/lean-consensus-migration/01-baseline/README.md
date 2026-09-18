# Baseline

This directory records the replacement starting point for the Ethean Lean Consensus migration.

## Contents

- [CURRENT_STATE_AUDIT.md](./CURRENT_STATE_AUDIT.md) states what the current repository does, which claims are unsupported, and which protocol/security gaps force replacement.
- [LEGACY_COMPONENT_MATRIX.md](./LEGACY_COMPONENT_MATRIX.md) inventories the current root, source, documentation, example, configuration, test, benchmark, and deployment surfaces with exact paths, owner phases, evidence, and removal gates.
- [REMOVAL_REWRITE_REUSE_MAP.md](./REMOVAL_REWRITE_REUSE_MAP.md) translates responsibilities into allowed replacement dispositions and enforces the no-retain rule.

## Baseline identity

- Repository revision observed: `880982f9635e8507e5cac37131c3696f6d06191b`.
- Peer/source research date: 2026-09-19.
- Product target: **Ethean Lean Consensus Client**.
- Baseline package identity: `panro`.

The working tree contained unrelated deletions/moves and a pre-existing `03-architecture/README.md`. This planning work does not resolve, restore, delete, or approve those changes.

## Inventory convention

- “Tracked” means present in Git's tracked-path inventory at the observed revision, even when a path was deleted in the working tree.
- “Current file” means readable from the working tree during the audit.
- A grouped matrix row still lists every exact path governed by that row.
- A missing top-level surface is explicitly recorded; it is not treated as implied test or deployment coverage.

## Disposition convention

No old implementation has retain status:

- **replace-then-delete** keeps an old path only until its new responsibility passes the named gate;
- **rename-by-replacement** creates a new Ethean identity/path and removes the old identity-bearing path;
- **delete** removes a path with no replacement responsibility.

Generic concepts such as a backend trait, middleware, queue bound, metric, or module boundary may inform new code. They do not permit the old file to survive.
