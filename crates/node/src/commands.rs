//! Typed commands toward the chain owner (bounded queues at the call site).

use ethean_primitives::Hash32;
use ethean_validator::DutyTick;

/// Commands that may mutate chain-owner state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainCommand {
    /// Clock tick from the injected scheduler.
    Tick(DutyTick),
    /// Import a locally verified block root.
    ImportBlock { root: Hash32, parent: Hash32 },
    /// Update syncing flag from the sync subsystem.
    SetSyncing(bool),
    /// Request graceful stop of new duties.
    Shutdown,
}
