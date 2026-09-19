//! Swarm event-loop facade (health + pending transport).

use crate::error::{NetworkError, Result};
use crate::peer_manager::PeerManager;
use crate::reqresp::RequestTracker;

/// High-level swarm state without an active QUIC runtime.
#[derive(Debug, Default)]
pub struct SwarmFacade {
    pub peers: PeerManager,
    pub requests: RequestTracker,
    pub events_processed: u64,
}

impl SwarmFacade {
    /// Record a tick of the (future) event loop for health.
    pub fn note_progress(&mut self) {
        self.events_processed = self.events_processed.saturating_add(1);
    }

    /// Healthy if the facade has observed progress since start (policy placeholder).
    pub fn is_progressing(&self) -> bool {
        self.events_processed > 0
    }

    /// Dial is unavailable until QUIC lands.
    pub fn dial_static_peer(&self, _multiaddr: &str) -> Result<()> {
        Err(NetworkError::TransportPending(
            "static dial requires QUIC transport",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_flag() {
        let mut s = SwarmFacade::default();
        assert!(!s.is_progressing());
        s.note_progress();
        assert!(s.is_progressing());
    }
}
