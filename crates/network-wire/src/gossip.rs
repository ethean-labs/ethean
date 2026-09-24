//! Gossipsub mesh parameters (leanSpec `gossipsub/parameters.py`).

use std::time::Duration;

use crate::limits::MAX_MESSAGE_SIZE;

/// Target mesh degree `D`.
pub const MESH_N: usize = 8;
/// Minimum mesh peers before grafting (`D_low`).
pub const MESH_N_LOW: usize = 6;
/// Maximum mesh peers before pruning (`D_high`).
pub const MESH_N_HIGH: usize = 12;
/// Non-mesh IHAVE gossip degree (`D_lazy`).
pub const GOSSIP_LAZY: usize = 6;
/// Heartbeat interval.
pub const HEARTBEAT: Duration = Duration::from_millis(700);
/// Fanout TTL.
pub const FANOUT_TTL: Duration = Duration::from_secs(60);
/// Message cache windows (`mcache_length`).
pub const HISTORY_LENGTH: usize = 6;
/// Windows included in IHAVE (`mcache_gossip`).
pub const HISTORY_GOSSIP: usize = 3;
/// Duplicate cache: `SECONDS_PER_SLOT * JUSTIFICATION_LOOKBACK_SLOTS * 2` (lstar: 4 * 3 * 2).
pub const SEEN_TTL: Duration = Duration::from_secs(24);
/// IDONTWANT size threshold (bytes).
pub const IDONTWANT_MESSAGE_SIZE_THRESHOLD: usize = 1000;
/// Max messages per RPC.
pub const MAX_MESSAGES_PER_RPC: usize = 500;

/// Maximum gossipsub transmit size (snappy-framed SignedBlock budget).
pub fn max_transmit_size() -> usize {
    MAX_MESSAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_match_leanspec() {
        assert_eq!(MESH_N, 8);
        assert_eq!(MESH_N_LOW, 6);
        assert_eq!(MESH_N_HIGH, 12);
        assert_eq!(GOSSIP_LAZY, 6);
        assert_eq!(HEARTBEAT, Duration::from_millis(700));
        assert_eq!(HISTORY_LENGTH, 6);
        assert_eq!(HISTORY_GOSSIP, 3);
        assert!(max_transmit_size() >= 1024 * 1024);
    }
}
