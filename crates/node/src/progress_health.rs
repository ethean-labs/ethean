//! Progress health: process liveness is not enough (wedged head / prover).
//!
//! `/healthz` stays process-alive. `/readyz` also fails when the head, the
//! prover thread, or a Type-2 job has been stuck for [`STALL_SLOTS`]. Gossip
//! idle is a gauge only (a quiet mesh is not a crash).

/// Wall-clock slots with no progress before a gate flips (8 × 4 s = 32 s).
pub const STALL_SLOTS: u64 = 8;

/// Inputs sampled once per metrics refresh.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProgressSample {
    pub current_slot: u64,
    pub head_slot: u64,
    pub peer_count: u64,
    pub gossip_rx_total: u64,
    pub prover_expected: bool,
    pub prover_alive: bool,
    pub block_in_flight: bool,
}

/// Latched progress bits after comparing consecutive samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressFlags {
    pub head_progress: bool,
    pub prover_alive: bool,
    pub block_proof_progress: bool,
    /// True when peers exist but no gossip arrived for [`STALL_SLOTS`].
    pub gossip_idle: bool,
}

impl Default for ProgressFlags {
    fn default() -> Self {
        Self {
            head_progress: true,
            prover_alive: true,
            block_proof_progress: true,
            gossip_idle: false,
        }
    }
}

/// Remembers the last head / gossip / Type-2 occupancy across refreshes.
#[derive(Debug, Default)]
pub struct ProgressTracker {
    last_head: Option<u64>,
    head_since: Option<u64>,
    last_gossip_rx: u64,
    gossip_since: Option<u64>,
    block_since: Option<u64>,
    flags: ProgressFlags,
}

impl ProgressTracker {
    /// Fold `sample` into the latches. First sample never stalls.
    pub fn observe(&mut self, sample: ProgressSample) -> ProgressFlags {
        self.flags.head_progress = self.head_ok(sample);
        self.flags.prover_alive = !sample.prover_expected || sample.prover_alive;
        self.flags.block_proof_progress = self.block_ok(sample);
        self.flags.gossip_idle = self.gossip_idle(sample);
        self.flags
    }

    pub fn flags(&self) -> ProgressFlags {
        self.flags
    }

    fn head_ok(&mut self, sample: ProgressSample) -> bool {
        if self.last_head != Some(sample.head_slot) {
            self.last_head = Some(sample.head_slot);
            self.head_since = Some(sample.current_slot);
            return true;
        }
        let since = self.head_since.unwrap_or(sample.current_slot);
        sample.current_slot.saturating_sub(since) < STALL_SLOTS
    }

    fn block_ok(&mut self, sample: ProgressSample) -> bool {
        if !sample.block_in_flight {
            self.block_since = None;
            return true;
        }
        if self.block_since.is_none() {
            self.block_since = Some(sample.current_slot);
            return true;
        }
        sample
            .current_slot
            .saturating_sub(self.block_since.unwrap_or(sample.current_slot))
            < STALL_SLOTS
    }

    fn gossip_idle(&mut self, sample: ProgressSample) -> bool {
        if sample.peer_count == 0 {
            self.last_gossip_rx = sample.gossip_rx_total;
            self.gossip_since = Some(sample.current_slot);
            return false;
        }
        if sample.gossip_rx_total != self.last_gossip_rx {
            self.last_gossip_rx = sample.gossip_rx_total;
            self.gossip_since = Some(sample.current_slot);
            return false;
        }
        let since = self.gossip_since.unwrap_or(sample.current_slot);
        sample.current_slot.saturating_sub(since) >= STALL_SLOTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(current: u64, head: u64) -> ProgressSample {
        ProgressSample {
            current_slot: current,
            head_slot: head,
            ..ProgressSample::default()
        }
    }

    #[test]
    fn head_stalls_after_eight_slots_without_a_move() {
        let mut t = ProgressTracker::default();
        assert!(t.observe(sample(1, 1)).head_progress);
        assert!(t.observe(sample(8, 1)).head_progress);
        assert!(!t.observe(sample(9, 1)).head_progress);
        assert!(t.observe(sample(10, 2)).head_progress);
    }

    #[test]
    fn a_dead_prover_thread_fails_when_a_worker_is_expected() {
        let mut t = ProgressTracker::default();
        let mut s = sample(1, 1);
        s.prover_expected = true;
        s.prover_alive = false;
        assert!(!t.observe(s).prover_alive);
        s.prover_expected = false;
        assert!(t.observe(s).prover_alive);
    }

    #[test]
    fn a_block_proof_stuck_for_eight_slots_fails() {
        let mut t = ProgressTracker::default();
        let mut s = sample(1, 1);
        s.block_in_flight = true;
        assert!(t.observe(s).block_proof_progress);
        s.current_slot = 9;
        assert!(!t.observe(s).block_proof_progress);
        s.block_in_flight = false;
        assert!(t.observe(s).block_proof_progress);
    }

    #[test]
    fn gossip_idle_needs_peers_and_a_quiet_window() {
        let mut t = ProgressTracker::default();
        let mut s = sample(1, 1);
        s.peer_count = 1;
        s.gossip_rx_total = 3;
        assert!(!t.observe(s).gossip_idle);
        s.current_slot = 9;
        assert!(t.observe(s).gossip_idle);
        s.peer_count = 0;
        assert!(!t.observe(s).gossip_idle);
    }
}
