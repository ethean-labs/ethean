//! Events emitted by the chain owner to workers and API observers.

use ethean_primitives::Hash32;
use ethean_validator::{DutyTick, SuppressReason};

/// Outbound events (non-mutating observers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainEvent {
    /// A new duty tick was accepted.
    TickAccepted(DutyTick),
    /// A duplicate tick was ignored.
    TickDuplicate(DutyTick),
    /// A duty was suppressed before signing.
    DutySuppressed { tick: DutyTick, reason: SuppressReason },
    /// Head advanced after local import.
    HeadUpdated { root: Hash32, slot: u64 },
    /// Validated gossip ingested (content root recorded; SSZ decode still open).
    GossipIngested {
        /// Topic path that carried the payload.
        topic: String,
        /// SHA-256 of the decompressed payload (provisional content id).
        content_root: Hash32,
    },
    /// Syncing flag changed on the chain owner.
    SyncingUpdated(bool),
    /// Shutdown acknowledged; no new duties.
    ShutdownComplete,
}
