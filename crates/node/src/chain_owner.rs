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
#[derive(Debug, Default)]
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
    /// Slot whose block proof was submitted and has not failed yet.
    pub block_proof_slot: Option<u64>,
    /// Block proof that finished before the wall clock reached its slot.
    pub deferred_block_proof: Option<crate::proof_collect::DeferredBlockProof>,
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
    /// Registry signers by validator index. When set they replace
    /// `attester` / `proposer`, which only serve solo and smoke runs.
    pub signers: std::collections::BTreeMap<u64, ValidatorSigners>,
    /// Keeps installed XMSS keys' windows prepared off the signing path.
    pub key_prep: crate::key_prep::KeyPreparer,
    /// Validator indices owned by this node (from Hive registry).
    pub owned_validator_indices: Vec<u64>,
    /// Sync blobs waiting for a missing parent (blocks-by-root catch-up).
    pub sync_orphans: SyncOrphanCache,
    /// Finalized slot the fork-choice store history was last pruned at.
    pub fc_pruned_finalized_slot: u64,
    /// Votes the store rejected because their block or tick had not arrived.
    pub deferred_votes: crate::fc_vote_retry::DeferredVotes,
    /// Configured max head lag.
    pub max_head_lag_slots: u64,
    /// Attestation data per proved block; `None` means
    /// [`crate::block_builder::DEFAULT_MAX_BLOCK_ATTESTATION_DATA`].
    pub max_block_attestation_data: Option<usize>,
    /// Collect/prove aggregates (Lean aggregator role).
    pub is_aggregator: bool,
    /// Extra subnets to aggregate (`--aggregate-subnet-ids`), on top of the
    /// subnets of `owned_validator_indices`.
    pub aggregate_subnet_ids: Vec<u64>,
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

/// Signing keys of one owned validator.
#[derive(Debug, Default)]
pub struct ValidatorSigners {
    pub attester: Option<LocalAttester>,
    pub proposer: Option<LocalProposer>,
}

impl ChainOwner {
    /// Validators this node signs attestations for.
    pub fn attesting_indices(&self) -> Vec<u64> {
        if !self.signers.is_empty() {
            return self
                .signers
                .iter()
                .filter(|(_, s)| s.attester.is_some())
                .map(|(i, _)| *i)
                .collect();
        }
        match (&self.attester, self.owned_validator_indices.first()) {
            (Some(_), Some(first)) => vec![*first],
            _ => Vec::new(),
        }
    }

    /// Attestation signer for `index`.
    pub fn attester_for(&mut self, index: u64) -> Option<&mut LocalAttester> {
        if self.signers.is_empty() {
            return self.attester.as_mut();
        }
        self.signers.get_mut(&index)?.attester.as_mut()
    }

    /// Proposal signer for `index`.
    pub fn proposer_for(&mut self, index: u64) -> Option<&mut LocalProposer> {
        if self.signers.is_empty() {
            return self.proposer.as_mut();
        }
        self.signers.get_mut(&index)?.proposer.as_mut()
    }

    pub fn has_attester(&self) -> bool {
        self.attester.is_some() || self.signers.values().any(|s| s.attester.is_some())
    }

    pub fn has_proposer(&self) -> bool {
        self.proposer.is_some() || self.signers.values().any(|s| s.proposer.is_some())
    }

    /// Construct with lag budget.
    pub fn new(max_head_lag_slots: u64) -> Self {
        Self {
            max_head_lag_slots,
            ..Self::default()
        }
    }

    /// Attestation data a proved block may carry.
    pub fn block_attestation_data_cap(&self) -> usize {
        self.max_block_attestation_data
            .unwrap_or(crate::block_builder::DEFAULT_MAX_BLOCK_ATTESTATION_DATA)
    }

    /// Whether an aggregator pools the vote of `validator_index`. leanSpec
    /// aggregators only see the subnets they subscribe to (their validators'
    /// subnets plus `--aggregate-subnet-ids`); Ethean relays every subnet, so
    /// the same scope is applied at pooling time. With no owned validators and
    /// no extra ids every vote is pooled.
    pub fn aggregates_vote_of(&self, validator_index: u64) -> bool {
        self.aggregates_subnet(validator_index % self.attestation_committees())
    }

    /// Whether `subnet` is in this node's aggregation scope (see
    /// [`Self::aggregates_vote_of`]).
    pub fn aggregates_subnet(&self, subnet: u64) -> bool {
        if self.owned_validator_indices.is_empty() && self.aggregate_subnet_ids.is_empty() {
            return true;
        }
        let committees = self.attestation_committees();
        self.aggregate_subnet_ids.contains(&subnet)
            || self
                .owned_validator_indices
                .iter()
                .any(|i| i % committees == subnet)
    }

    /// `ATTESTATION_COMMITTEE_COUNT` of the loaded profile (1 without one).
    pub fn attestation_committees(&self) -> u64 {
        self.profile
            .as_ref()
            .map(|p| p.attestation_committee_count.max(1))
            .unwrap_or(1)
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
    fn aggregator_pools_only_its_subnets() {
        let mut owner = ChainOwner::new(0);
        assert!(owner.aggregates_vote_of(5), "no scope: pool everything");
        owner.profile = Some(
            crate::lstar_devnet()
                .unwrap()
                .with_attestation_committee_count(3)
                .unwrap(),
        );
        owner.owned_validator_indices = vec![1, 4];
        assert!(owner.aggregates_vote_of(7));
        assert!(!owner.aggregates_vote_of(6));
        assert!(!owner.aggregates_vote_of(8));
        owner.aggregate_subnet_ids = vec![2];
        assert!(owner.aggregates_vote_of(8));
        assert!(!owner.aggregates_vote_of(6));
    }

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
