//! Chain owner: sole writer of transition, fork choice, and import status.

use crate::aggregation::{AggregatePool, AttestationSignaturePool};
use crate::aggregation_gossip::AggregationGossip;
use crate::block_builder::{PlanTransition, ProposalGossip};
use crate::local_attester::LocalAttester;
use crate::local_proposer::LocalProposer;
use crate::proof_service::ProofService;
use crate::sync_orphan::SyncOrphanCache;
use ethean_primitives::{Hash32, Slot};
use ethean_profile::ChainProfile;
use ethean_types::State;
use ethean_validator::{DutyTick, DutyView};

/// Immutable worker snapshot published by the chain owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainSnapshot {
    /// Scheduler generation at snapshot time.
    pub generation: u64,
    /// Wall slot.
    pub wall_slot: Slot,
    /// Canonical head root.
    pub head_root: Hash32,
    /// Parent root for the next proposal (usually head).
    pub parent_root: Hash32,
    /// Safe target root from fork choice.
    pub safe_target: Hash32,
    /// Justified checkpoint root.
    pub justified_root: Hash32,
    /// Finalized checkpoint root.
    pub finalized_root: Hash32,
    /// Duty view for the gate.
    pub duty_view: DutyView,
}

/// Owns chain mutation; workers only read snapshots / send commands.
#[derive(Debug)]
pub struct ChainOwner {
    /// Current generation for stale rejection.
    pub generation: u64,
    /// Last applied tick.
    pub last_tick: Option<DutyTick>,
    /// Latest head root.
    pub head_root: Hash32,
    /// Post-state of head (optional until storage wiring).
    pub head_state: Option<State>,
    /// Syncing flag from sync subsystem.
    pub syncing: bool,
    /// Last ingested gossip content root (provisional until SSZ import).
    pub last_gossip_root: Option<Hash32>,
    /// Chain profile for structural gossip state transitions.
    pub profile: Option<ChainProfile>,
    /// Verified Type-1 proofs from aggregation gossip and local aggregation.
    pub aggregates: AggregatePool,
    /// Verified individual votes awaiting aggregation (aggregators only).
    pub signatures: AttestationSignaturePool,
    /// Background leanMultisig prover, when an `ethean-prover` binary is available.
    pub prover: Option<ProofService>,
    /// Last locally planned proposal (cleared when superseded).
    pub planned_proposal: Option<PlanTransition>,
    /// Tick that produced `planned_proposal`, if any.
    pub planned_tick: Option<DutyTick>,
    /// Encoded SignedBlock waiting for network gossip publish.
    pub pending_block_gossip: Option<ProposalGossip>,
    /// Recently applied block SSZ blobs waiting for `--data-dir` flush.
    pub durable_blocks: Vec<(Hash32, Vec<u8>)>,
    /// Votes and aggregates waiting for attestation-subnet / aggregation publish.
    pub pending_aggregation_gossip: Vec<AggregationGossip>,
    /// Optional local proposal signer (test-hmac smoke key).
    pub proposer: Option<LocalProposer>,
    /// Optional local attestation signer (Hive registry / smoke).
    pub attester: Option<LocalAttester>,
    /// Validator indices owned by this node (from Hive registry).
    pub owned_validator_indices: Vec<u64>,
    /// Sync blobs waiting for a missing parent (blocks-by-root catch-up).
    pub sync_orphans: SyncOrphanCache,
    /// Configured max head lag.
    pub max_head_lag_slots: u64,
    /// Collect/prove aggregates (Lean aggregator role).
    pub is_aggregator: bool,
    /// Self-apply proposals + inject full-registry votes for local finality smoke.
    pub local_finality: bool,
    /// Safe-target root (from FC store when live; else justified).
    pub safe_target: Hash32,
    /// Head moves onto a competing branch (leanMetrics `fc_reorg_total`).
    pub reorg_total: u64,
    /// Live lstar fork-choice store (structural until proofs are required in-node).
    pub fc: Option<ethean_fork_choice::ForkChoiceStore>,
    /// Attestation data seen on chain, by data root -> vote slot
    /// (leanSpec `latest_known_aggregated_payloads`).
    pub known_payloads: std::collections::HashMap<Hash32, u64>,
}

impl Default for ChainOwner {
    fn default() -> Self {
        Self {
            generation: 0,
            last_tick: None,
            head_root: Hash32::default(),
            head_state: None,
            syncing: false,
            last_gossip_root: None,
            profile: None,
            aggregates: AggregatePool::default(),
            signatures: AttestationSignaturePool::default(),
            prover: None,
            planned_proposal: None,
            planned_tick: None,
            pending_block_gossip: None,
            durable_blocks: Vec::new(),
            pending_aggregation_gossip: Vec::new(),
            proposer: None,
            attester: None,
            owned_validator_indices: Vec::new(),
            sync_orphans: SyncOrphanCache::default(),
            max_head_lag_slots: 0,
            is_aggregator: false,
            local_finality: false,
            safe_target: Hash32::default(),
            reorg_total: 0,
            fc: None,
            known_payloads: std::collections::HashMap::new(),
        }
    }
}

impl ChainOwner {
    /// Construct with lag budget.
    pub fn new(max_head_lag_slots: u64) -> Self {
        Self {
            max_head_lag_slots,
            ..Self::default()
        }
    }

    /// Apply a clock tick; returns false when duplicate.
    pub fn on_tick(&mut self, tick: DutyTick) -> bool {
        if !ethean_validator::should_process(self.last_tick, tick) {
            return false;
        }
        self.last_tick = Some(tick);
        true
    }

    /// Bump generation after restart / catch-up.
    pub fn bump_generation(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }

    /// Build a snapshot for workers (stale if generation mismatches later).
    pub fn snapshot(&self, wall_slot: Slot, head_lag_slots: u64) -> ChainSnapshot {
        let duty_view = DutyView {
            wall_slot,
            genesis_slot: Slot::new(0),
            syncing: self.syncing,
            parent_state_available: self.head_state.is_some() || wall_slot.get() == 0,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots,
            max_head_lag_slots: self.max_head_lag_slots,
        };
        let (justified_root, finalized_root) = self
            .head_state
            .as_ref()
            .map(|s| (s.latest_justified.root, s.latest_finalized.root))
            .unwrap_or((self.head_root, self.head_root));
        ChainSnapshot {
            generation: self.generation,
            wall_slot,
            head_root: self.head_root,
            parent_root: self.head_root,
            safe_target: self.safe_target,
            justified_root,
            finalized_root,
            duty_view,
        }
    }

    /// Reject a worker result when generation or parent drifted.
    pub fn accept_result(&self, generation: u64, parent_root: Hash32) -> bool {
        generation == self.generation && parent_root == self.head_root
    }

    /// Queue a block blob for durable flush (capped; newest wins on duplicate root).
    pub fn remember_durable_block(&mut self, root: Hash32, payload: Vec<u8>) {
        const CAP: usize = 64;
        if payload.is_empty() {
            return;
        }
        if let Some(slot) = self.durable_blocks.iter().position(|(r, _)| *r == root) {
            self.durable_blocks[slot] = (root, payload);
            return;
        }
        self.durable_blocks.push((root, payload));
        while self.durable_blocks.len() > CAP {
            self.durable_blocks.remove(0);
        }
    }

    /// Drop queued durable blobs after a successful data-dir flush.
    pub fn clear_durable_blocks(&mut self) {
        self.durable_blocks.clear();
    }

    /// Look up a remembered SignedBlock SSZ blob by root.
    pub fn durable_block(&self, root: &Hash32) -> Option<&[u8]> {
        self.durable_blocks
            .iter()
            .find(|(r, _)| r == root)
            .map(|(_, p)| p.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupes_ticks() {
        let mut owner = ChainOwner::new(2);
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        assert!(owner.on_tick(tick));
        assert!(!owner.on_tick(tick));
    }

    #[test]
    fn rejects_stale_generation() {
        let mut owner = ChainOwner::new(2);
        owner.head_root = [9u8; 32];
        owner.bump_generation();
        assert!(!owner.accept_result(0, [9u8; 32]));
        assert!(owner.accept_result(1, [9u8; 32]));
    }
}
