//! Sync status and hysteretic duty gate.

use ethean_primitives::Slot;

/// Whether the node may run validator duties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Catching up; duties suppressed.
    Syncing,
    /// Within lag budget; duties allowed.
    Synced,
}

/// Hysteretic sync/duty gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncStatus {
    pub mode: SyncMode,
    pub local_head: Slot,
    pub peer_horizon: Slot,
    /// Enter syncing when lag > high.
    pub lag_high: u64,
    /// Return to synced when lag ≤ low.
    pub lag_low: u64,
}

impl SyncStatus {
    /// Construct in Syncing with default hysteresis (high=8, low=2).
    pub fn new(local_head: Slot, peer_horizon: Slot) -> Self {
        Self {
            mode: SyncMode::Syncing,
            local_head,
            peer_horizon,
            lag_high: 8,
            lag_low: 2,
        }
    }

    /// Current lag in slots.
    pub fn lag(&self) -> u64 {
        self.peer_horizon
            .get()
            .saturating_sub(self.local_head.get())
    }

    /// Update heads and apply hysteresis.
    pub fn observe(&mut self, local_head: Slot, peer_horizon: Slot) {
        self.local_head = local_head;
        self.peer_horizon = peer_horizon;
        let lag = self.lag();
        match self.mode {
            SyncMode::Synced if lag > self.lag_high => self.mode = SyncMode::Syncing,
            SyncMode::Syncing if lag <= self.lag_low => self.mode = SyncMode::Synced,
            _ => {}
        }
    }

    /// True when duties may run.
    pub fn duties_allowed(&self) -> bool {
        self.mode == SyncMode::Synced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hysteresis() {
        let mut s = SyncStatus::new(Slot::new(0), Slot::new(20));
        assert!(!s.duties_allowed());
        s.observe(Slot::new(19), Slot::new(20));
        assert!(s.duties_allowed());
        s.observe(Slot::new(19), Slot::new(40));
        assert!(!s.duties_allowed());
    }
}
