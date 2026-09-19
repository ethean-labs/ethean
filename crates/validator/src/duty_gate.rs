//! Duty gate: suppress signing without a trustworthy chain view.

use ethean_primitives::Slot;

/// Default max head lag before duties are suppressed (leanSpec / PQ Interop sync-lag gate).
///
/// Interop #37 / leanSpec #689 direction: skip attestation and proposal when local head
/// trails wall clock by more than this many slots.
pub const SYNC_LAG_THRESHOLD_SLOTS: u64 = 4;

/// Reasons a duty must not sign or publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressReason {
    /// Wall time is before genesis.
    PreGenesis,
    /// Node is still syncing toward a trustworthy head.
    Syncing,
    /// Parent / head state is missing locally.
    MissingParentState,
    /// Active profile does not match the chain fingerprint.
    ProfileMismatch,
    /// Signer journal is unsafe (exhausted, burned, or not flushed).
    SignerUnsafe,
    /// Local head lags beyond the configured horizon.
    HeadLagExceeded,
}

/// Immutable snapshot inputs for the gate (filled by the chain owner).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DutyView {
    /// Current wall slot from the injected clock.
    pub wall_slot: Slot,
    /// Genesis slot (usually 0).
    pub genesis_slot: Slot,
    /// True while sync has not reached a trustworthy lag.
    pub syncing: bool,
    /// True when the parent block state is available.
    pub parent_state_available: bool,
    /// True when profile fingerprint matches the loaded chain.
    pub profile_matches: bool,
    /// True when the local signer can accept a reservation.
    pub signer_safe: bool,
    /// Slots between wall time and local head (0 = caught up).
    pub head_lag_slots: u64,
    /// Maximum allowed head lag before suppression.
    pub max_head_lag_slots: u64,
}

/// Evaluate whether duties may proceed for this view.
pub fn evaluate_gate(view: &DutyView) -> Result<(), SuppressReason> {
    if view.wall_slot.get() < view.genesis_slot.get() {
        return Err(SuppressReason::PreGenesis);
    }
    if view.syncing {
        return Err(SuppressReason::Syncing);
    }
    if !view.parent_state_available {
        return Err(SuppressReason::MissingParentState);
    }
    if !view.profile_matches {
        return Err(SuppressReason::ProfileMismatch);
    }
    if !view.signer_safe {
        return Err(SuppressReason::SignerUnsafe);
    }
    if view.head_lag_slots > view.max_head_lag_slots {
        return Err(SuppressReason::HeadLagExceeded);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_view() -> DutyView {
        DutyView {
            wall_slot: Slot::new(10),
            genesis_slot: Slot::new(0),
            syncing: false,
            parent_state_available: true,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots: 0,
            max_head_lag_slots: SYNC_LAG_THRESHOLD_SLOTS,
        }
    }

    #[test]
    fn allows_healthy_view() {
        assert!(evaluate_gate(&ok_view()).is_ok());
    }

    #[test]
    fn suppresses_syncing() {
        let mut v = ok_view();
        v.syncing = true;
        assert_eq!(evaluate_gate(&v), Err(SuppressReason::Syncing));
    }

    #[test]
    fn suppresses_head_lag() {
        let mut v = ok_view();
        v.head_lag_slots = SYNC_LAG_THRESHOLD_SLOTS.saturating_add(1);
        assert_eq!(evaluate_gate(&v), Err(SuppressReason::HeadLagExceeded));
    }

    #[test]
    fn allows_lag_at_threshold() {
        let mut v = ok_view();
        v.head_lag_slots = SYNC_LAG_THRESHOLD_SLOTS;
        assert!(evaluate_gate(&v).is_ok());
    }
}
