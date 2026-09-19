# ethean-node

Node library crate. Holds legacy modules moved from the former root `src/` during Phase 02 workspace migration.

Depends on `ethean-primitives`, `ethean-profile`, `ethean-types`, and `ethean-genesis`. Slot clock and genesis startup live under `src/clock.rs` / `src/client.rs`. The production binary lives in [`../../bin/ethean`](../../bin/ethean/).
