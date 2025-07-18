//! GRANDPA-style finality gadget for Beam Chain
//!
//! Implements a finality mechanism that provides deterministic finality
//! through a separate voting process independent of block production.

use crate::types::{BeaconState, Epoch, ValidatorIndex};
use crate::crypto::hash::Hash;
use crate::consensus::validator_management::ValidatorManager;
use crate::crypto::bls::{RealBLSAggregator, BLSSignature, BLSPublicKey, BLSError};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::{HashMap, HashSet, BTreeMap};
use std::time::{Duration, Instant};

/// Finality errors
#[derive(Debug, Error)]
pub enum FinalityError {
    #[error("Invalid finality vote from validator {validator} for round {round}")]
    InvalidVote { validator: ValidatorIndex, round: u64 },
    
    #[error("Conflicting finality votes detected for validator {validator}")]
    ConflictingVotes { validator: ValidatorIndex },
    
    #[error("Insufficient voting power: {current}/{required}")]
    InsufficientVotingPower { current: u64, required: u64 },
    
    #[error("Invalid checkpoint: epoch {epoch}, block {block_hash:?}")]
    InvalidCheckpoint { epoch: Epoch, block_hash: Hash },
    
    #[error("Finality safety violation detected")]
    SafetyViolation,
    
    #[error("Vote verification failed: {0}")]
    VoteVerification(#[from] BLSError),
}

/// Finality vote types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    /// Pre-vote for a block
    Prevote,
    /// Pre-commit for a block  
    Precommit,
}

/// A finality vote from a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityVote {
    /// Voting round number
    pub round: u64,
    /// Type of vote (prevote or precommit)
    pub vote_type: VoteType,
    /// Target checkpoint being voted for
    pub target: Checkpoint,
    /// Validator who cast the vote
    pub validator: ValidatorIndex,
    /// BLS signature of the vote
    pub signature: BLSSignature,
    /// Timestamp when vote was created
    pub timestamp: u64,
}

/// A checkpoint representing a finalized state
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Epoch number
    pub epoch: Epoch,
    /// Block hash at this checkpoint
    pub block_hash: Hash,
    /// State root at this checkpoint
    pub state_root: Hash,
}

/// Aggregated votes for a specific checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteSet {
    /// Target checkpoint
    pub target: Checkpoint,
    /// Round number
    pub round: u64,
    /// Vote type
    pub vote_type: VoteType,
    /// Participating validators
    pub validators: HashSet<ValidatorIndex>,
    /// Aggregated signature
    pub signature: Option<BLSSignature>,
    /// Total voting weight
    pub weight: u64,
    /// Timestamp of first vote
    pub first_vote_time: u64,
}

/// Finality round state
#[derive(Debug, Clone)]
pub struct FinalityRound {
    /// Round number
    pub round: u64,
    /// Prevotes collected this round
    pub prevotes: HashMap<Checkpoint, VoteSet>,
    /// Precommits collected this round
    pub precommits: HashMap<Checkpoint, VoteSet>,
    /// Round start time (as timestamp)
    pub start_time: u64,
    /// Whether this round has completed
    pub completed: bool,
}

/// Finality tracker for monitoring justified and finalized checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityTracker {
    /// Latest justified checkpoint
    pub justified_checkpoint: Checkpoint,
    /// Latest finalized checkpoint
    pub finalized_checkpoint: Checkpoint,
    /// Finalization history
    pub finalized_history: BTreeMap<Epoch, Checkpoint>,
    /// Justification votes tracking
    pub justification_votes: HashMap<Checkpoint, u64>,
}

impl FinalityTracker {
    /// Create new finality tracker
    pub fn new(genesis_checkpoint: Checkpoint) -> Self {
        let mut finalized_history = BTreeMap::new();
        finalized_history.insert(genesis_checkpoint.epoch, genesis_checkpoint.clone());
        
        Self {
            justified_checkpoint: genesis_checkpoint.clone(),
            finalized_checkpoint: genesis_checkpoint,
            finalized_history,
            justification_votes: HashMap::new(),
        }
    }
    
    /// Update justified checkpoint
    pub fn update_justified(&mut self, checkpoint: Checkpoint, voting_weight: u64) -> Result<(), FinalityError> {
        // Only update if this is a newer checkpoint
        if checkpoint.epoch > self.justified_checkpoint.epoch {
            self.justified_checkpoint = checkpoint.clone();
            self.justification_votes.insert(checkpoint, voting_weight);
        }
        Ok(())
    }
    
    /// Update finalized checkpoint
    pub fn update_finalized(&mut self, checkpoint: Checkpoint) -> Result<(), FinalityError> {
        // Ensure finalized checkpoint advances monotonically
        if checkpoint.epoch <= self.finalized_checkpoint.epoch {
            return Err(FinalityError::SafetyViolation);
        }
        
        let checkpoint_epoch = checkpoint.epoch;
        
        // Update finalized checkpoint
        self.finalized_checkpoint = checkpoint.clone();
        self.finalized_history.insert(checkpoint_epoch, checkpoint);
        
        // Clean old justification votes
        self.justification_votes.retain(|cp, _| cp.epoch >= checkpoint_epoch);
        
        Ok(())
    }
    
    /// Check if a checkpoint is finalized
    pub fn is_finalized(&self, checkpoint: &Checkpoint) -> bool {
        checkpoint.epoch <= self.finalized_checkpoint.epoch &&
        self.finalized_history.get(&checkpoint.epoch) == Some(checkpoint)
    }
    
    /// Check if a checkpoint is justified
    pub fn is_justified(&self, checkpoint: &Checkpoint) -> bool {
        checkpoint == &self.justified_checkpoint ||
        self.is_finalized(checkpoint)
    }
    
    /// Get finalization lag (epochs behind head)
    pub fn get_finality_lag(&self, head_epoch: Epoch) -> u64 {
        head_epoch.saturating_sub(self.finalized_checkpoint.epoch)
    }
}

/// Vote aggregator for collecting and validating finality votes
#[derive(Debug)]
pub struct VoteAggregator {
    /// BLS signature aggregator
    bls_aggregator: RealBLSAggregator,
    /// Current voting rounds
    active_rounds: HashMap<u64, FinalityRound>,
    /// Vote verification cache
    verification_cache: HashMap<Hash, bool>,
    /// Performance metrics
    votes_processed: u64,
    votes_verified: u64,
    votes_rejected: u64,
}

impl VoteAggregator {
    /// Create new vote aggregator
    pub fn new() -> Self {
        Self {
            bls_aggregator: RealBLSAggregator::new(),
            active_rounds: HashMap::new(),
            verification_cache: HashMap::new(),
            votes_processed: 0,
            votes_verified: 0,
            votes_rejected: 0,
        }
    }
    
    /// Add a finality vote
    pub fn add_vote(
        &mut self,
        vote: FinalityVote,
        validator_manager: &ValidatorManager,
        state: &BeaconState,
    ) -> Result<(), FinalityError> {
        self.votes_processed += 1;
        
        // Verify vote signature
        if !self.verify_vote_signature(&vote, validator_manager, state)? {
            self.votes_rejected += 1;
            return Err(FinalityError::InvalidVote {
                validator: vote.validator,
                round: vote.round,
            });
        }
        
        self.votes_verified += 1;
        
        // Get current timestamp
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Get or create round
        let round = self.active_rounds
            .entry(vote.round)
            .or_insert_with(|| FinalityRound {
                round: vote.round,
                prevotes: HashMap::new(),
                precommits: HashMap::new(),
                start_time: current_time,
                completed: false,
            });
        
        // Add vote to appropriate vote set
        match vote.vote_type {
            VoteType::Prevote => {
                Self::add_to_vote_set_static(&mut round.prevotes, vote, validator_manager)?;
            },
            VoteType::Precommit => {
                Self::add_to_vote_set_static(&mut round.precommits, vote, validator_manager)?;
            },
        }
        
        Ok(())
    }
    
    /// Add vote to a vote set
    fn add_to_vote_set_static(
        vote_sets: &mut HashMap<Checkpoint, VoteSet>,
        vote: FinalityVote,
        validator_manager: &ValidatorManager,
    ) -> Result<(), FinalityError> {
        let vote_set = vote_sets
            .entry(vote.target.clone())
            .or_insert_with(|| VoteSet {
                target: vote.target.clone(),
                round: vote.round,
                vote_type: vote.vote_type,
                validators: HashSet::new(),
                signature: None,
                weight: 0,
                first_vote_time: vote.timestamp,
            });
        
        // Check for duplicate vote from same validator
        if vote_set.validators.contains(&vote.validator) {
            return Err(FinalityError::ConflictingVotes {
                validator: vote.validator,
            });
        }
        
        // Add validator to vote set
        vote_set.validators.insert(vote.validator);
        
        // Update voting weight
        let validator_weight = validator_manager
            .get_effective_balance(vote.validator)
            .unwrap_or(0) / 1_000_000_000; // Convert to ETH units
        vote_set.weight += validator_weight;
        
        // Aggregate signature (simplified - in production would properly aggregate)
        if vote_set.signature.is_none() {
            vote_set.signature = Some(vote.signature);
        }
        
        Ok(())
    }
    
    /// Verify vote signature
    fn verify_vote_signature(
        &mut self,
        vote: &FinalityVote,
        validator_manager: &ValidatorManager,
        _state: &BeaconState,
    ) -> Result<bool, FinalityError> {
        // Create vote hash for verification
        let vote_hash = self.create_vote_hash(vote);
        
        // Check cache first
        if let Some(&cached_result) = self.verification_cache.get(&vote_hash) {
            return Ok(cached_result);
        }
        
        // Create mock public key for validation (in production, would use real validator pubkey)
        let public_key = self.create_validator_public_key(vote.validator)?;
        
        // Verify signature
        let message = self.create_vote_message(vote)?;
        let result = self.bls_aggregator.verify_signature(&vote.signature, &message, &public_key)?;
        
        // Cache result
        self.verification_cache.insert(vote_hash, result);
        
        Ok(result)
    }
    
    /// Create vote hash for caching
    fn create_vote_hash(&self, vote: &FinalityVote) -> Hash {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(vote.round.to_le_bytes());
        hasher.update(&[vote.vote_type as u8]);
        hasher.update(vote.target.epoch.to_le_bytes());
        hasher.update(&vote.target.block_hash);
        hasher.update(vote.validator.to_le_bytes());
        hasher.update(vote.timestamp.to_le_bytes());
        
        hasher.finalize().into()
    }
    
    /// Create vote message for signing
    fn create_vote_message(&self, vote: &FinalityVote) -> Result<Vec<u8>, FinalityError> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(b"BEAM_FINALITY_VOTE");
        hasher.update(vote.round.to_le_bytes());
        hasher.update(&[vote.vote_type as u8]);
        hasher.update(vote.target.epoch.to_le_bytes());
        hasher.update(&vote.target.block_hash);
        
        Ok(hasher.finalize().to_vec())
    }
    
    /// Create mock validator public key
    fn create_validator_public_key(&self, validator_index: ValidatorIndex) -> Result<BLSPublicKey, FinalityError> {
        use bls12_381::{G2Projective, Scalar};
        
        // Create deterministic public key
        let scalar = Scalar::from(validator_index + 1000); // Offset for finality keys
        let point = G2Projective::generator() * scalar;
        
        Ok(BLSPublicKey::from_g2(&point.into()))
    }
    
    /// Get vote set for checkpoint and round
    pub fn get_vote_set(&self, round: u64, vote_type: VoteType, target: &Checkpoint) -> Option<&VoteSet> {
        let round_data = self.active_rounds.get(&round)?;
        
        match vote_type {
            VoteType::Prevote => round_data.prevotes.get(target),
            VoteType::Precommit => round_data.precommits.get(target),
        }
    }
    
    /// Check if threshold is met for finality
    pub fn check_finality_threshold(&self, vote_set: &VoteSet, total_weight: u64) -> bool {
        // Require 2/3+ voting weight for finality
        vote_set.weight * 3 > total_weight * 2
    }
    
    /// Cleanup old rounds
    pub fn cleanup_old_rounds(&mut self, current_round: u64) {
        let keep_rounds = 10; // Keep last 10 rounds
        self.active_rounds.retain(|&round, _| round + keep_rounds >= current_round);
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (self.votes_processed, self.votes_verified, self.votes_rejected)
    }
}

/// Main GRANDPA-style finality gadget
#[derive(Debug)]
pub struct FinalityGadget {
    /// Finality tracker
    tracker: FinalityTracker,
    /// Vote aggregator
    vote_aggregator: VoteAggregator,
    /// Current finality round
    current_round: u64,
    /// Validator manager reference
    validator_manager: ValidatorManager,
    /// Performance metrics
    rounds_completed: u64,
    checkpoints_finalized: u64,
    average_finality_time: Duration,
}

impl FinalityGadget {
    /// Create new finality gadget
    pub fn new(genesis_checkpoint: Checkpoint, validator_manager: ValidatorManager) -> Self {
        Self {
            tracker: FinalityTracker::new(genesis_checkpoint),
            vote_aggregator: VoteAggregator::new(),
            current_round: 0,
            validator_manager,
            rounds_completed: 0,
            checkpoints_finalized: 0,
            average_finality_time: Duration::ZERO,
        }
    }
    
    /// Process a finality vote
    pub fn process_vote(&mut self, vote: FinalityVote, state: &BeaconState) -> Result<(), FinalityError> {
        self.vote_aggregator.add_vote(vote, &self.validator_manager, state)
    }
    
    /// Try to finalize checkpoints based on current votes
    pub fn try_finalize(&mut self, state: &BeaconState) -> Result<Option<Checkpoint>, FinalityError> {
        let total_weight = self.calculate_total_voting_weight(state);
        
        // Check for finality in current and recent rounds
        for round_offset in 0..5 {
            let round = self.current_round.saturating_sub(round_offset);
            
            if let Some(finalized_checkpoint) = self.check_round_finality(round, total_weight)? {
                self.tracker.update_finalized(finalized_checkpoint.clone())?;
                self.checkpoints_finalized += 1;
                return Ok(Some(finalized_checkpoint));
            }
        }
        
        Ok(None)
    }
    
    /// Check if a round has achieved finality
    fn check_round_finality(&self, round: u64, total_weight: u64) -> Result<Option<Checkpoint>, FinalityError> {
        let round_data = match self.vote_aggregator.active_rounds.get(&round) {
            Some(round) => round,
            None => return Ok(None),
        };
        
        // Check each checkpoint for sufficient precommits
        for (checkpoint, vote_set) in &round_data.precommits {
            if self.vote_aggregator.check_finality_threshold(vote_set, total_weight) {
                // Also check if we have sufficient prevotes for the same checkpoint
                if let Some(prevote_set) = round_data.prevotes.get(checkpoint) {
                    if self.vote_aggregator.check_finality_threshold(prevote_set, total_weight) {
                        return Ok(Some(checkpoint.clone()));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Calculate total voting weight in current epoch
    fn calculate_total_voting_weight(&self, state: &BeaconState) -> u64 {
        state.validators.validators
            .iter()
            .enumerate()
            .filter(|(_, validator)| !validator.slashed)
            .map(|(_, validator)| validator.effective_balance / 1_000_000_000) // Convert to ETH
            .sum()
    }
    
    /// Advance to next finality round
    pub fn advance_round(&mut self) {
        self.current_round += 1;
        self.rounds_completed += 1;
        
        // Cleanup old rounds
        self.vote_aggregator.cleanup_old_rounds(self.current_round);
    }
    
    /// Get current finalized checkpoint
    pub fn get_finalized_checkpoint(&self) -> &Checkpoint {
        &self.tracker.finalized_checkpoint
    }
    
    /// Get current justified checkpoint
    pub fn get_justified_checkpoint(&self) -> &Checkpoint {
        &self.tracker.justified_checkpoint
    }
    
    /// Check if a checkpoint is finalized
    pub fn is_finalized(&self, checkpoint: &Checkpoint) -> bool {
        self.tracker.is_finalized(checkpoint)
    }
    
    /// Get finality lag
    pub fn get_finality_lag(&self, head_epoch: Epoch) -> u64 {
        self.tracker.get_finality_lag(head_epoch)
    }
    
    /// Get current round number
    pub fn get_current_round(&self) -> u64 {
        self.current_round
    }
    
    /// Get finality statistics
    pub fn get_finality_stats(&self) -> (u64, u64, Duration) {
        (self.rounds_completed, self.checkpoints_finalized, self.average_finality_time)
    }
}

impl Default for VoteAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{StateStore, Database};
    use crate::types::{BeaconState, Validator};
    
    fn setup_test_environment() -> (FinalityGadget, BeaconState) {
        let genesis_checkpoint = Checkpoint {
            epoch: 0,
            block_hash: [0u8; 32],
            state_root: [0u8; 32],
        };
        
        let validator_config = ValidatorConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database);
        let validator_manager = ValidatorManager::new(validator_config, state_store);
        
        let gadget = FinalityGadget::new(genesis_checkpoint, validator_manager);
        
        let mut state = BeaconState::default();
        // Add test validators
        for i in 0..10 {
            let validator = Validator {
                pubkey: vec![i as u8; 48],
                withdrawal_credentials: [i as u8; 32],
                effective_balance: 32_000_000_000, // 32 ETH
                slashed: false,
                activation_epoch: 0,
                exit_epoch: u64::MAX,
            };
            state.validators.validators.push(validator);
        }
        
        (gadget, state)
    }
    
    fn create_test_vote(validator: ValidatorIndex, round: u64, vote_type: VoteType) -> FinalityVote {
        use bls12_381::{G1Projective, Scalar};
        
        let checkpoint = Checkpoint {
            epoch: round / 10, // Simplified epoch calculation
            block_hash: [round as u8; 32],
            state_root: [round as u8; 32],
        };
        
        // Create mock signature
        let scalar = Scalar::from(validator + round + 1);
        let point = G1Projective::generator() * scalar;
        let signature = BLSSignature::from_g1(&point.into());
        
        FinalityVote {
            round,
            vote_type,
            target: checkpoint,
            validator,
            signature,
            timestamp: round * 1000, // Mock timestamp
        }
    }
    
    #[test]
    fn test_finality_gadget_creation() {
        let (gadget, _) = setup_test_environment();
        
        assert_eq!(gadget.get_current_round(), 0);
        assert_eq!(gadget.get_finalized_checkpoint().epoch, 0);
        assert_eq!(gadget.get_finality_lag(10), 10);
    }
    
    #[test]
    fn test_finality_tracker() {
        let genesis = Checkpoint {
            epoch: 0,
            block_hash: [0u8; 32],
            state_root: [0u8; 32],
        };
        
        let mut tracker = FinalityTracker::new(genesis.clone());
        
        assert!(tracker.is_finalized(&genesis));
        assert!(tracker.is_justified(&genesis));
        assert_eq!(tracker.get_finality_lag(5), 5);
        
        // Test justification update
        let justified = Checkpoint {
            epoch: 2,
            block_hash: [2u8; 32],
            state_root: [2u8; 32],
        };
        
        let result = tracker.update_justified(justified.clone(), 1000);
        assert!(result.is_ok());
        assert!(tracker.is_justified(&justified));
    }
    
    #[test]
    fn test_vote_aggregator() {
        let mut aggregator = VoteAggregator::new();
        let (_, state) = setup_test_environment();
        
        let validator_config = ValidatorConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database);
        let validator_manager = ValidatorManager::new(validator_config, state_store);
        
        // Test vote processing
        let vote = create_test_vote(0, 1, VoteType::Prevote);
        let result = aggregator.add_vote(vote, &validator_manager, &state);
        assert!(result.is_ok());
        
        let (processed, verified, rejected) = aggregator.get_stats();
        assert_eq!(processed, 1);
        assert_eq!(verified, 1);
        assert_eq!(rejected, 0);
    }
    
    #[test]
    fn test_vote_verification() {
        let mut aggregator = VoteAggregator::new();
        let vote = create_test_vote(0, 1, VoteType::Prevote);
        
        let vote_hash = aggregator.create_vote_hash(&vote);
        assert_eq!(vote_hash.len(), 32); // SHA256 hash length
        
        let message = aggregator.create_vote_message(&vote);
        assert!(message.is_ok());
        assert!(!message.unwrap().is_empty());
    }
    
    #[test]
    fn test_finality_threshold() {
        let aggregator = VoteAggregator::new();
        
        let vote_set = VoteSet {
            target: Checkpoint {
                epoch: 1,
                block_hash: [1u8; 32],
                state_root: [1u8; 32],
            },
            round: 1,
            vote_type: VoteType::Precommit,
            validators: HashSet::new(),
            signature: None,
            weight: 67, // 67% of total weight
            first_vote_time: 1000,
        };
        
        // Should reach threshold with 67% when requiring 2/3
        assert!(aggregator.check_finality_threshold(&vote_set, 100));
        
        // Should not reach threshold with 65%
        let low_vote_set = VoteSet {
            weight: 65,
            ..vote_set
        };
        assert!(!aggregator.check_finality_threshold(&low_vote_set, 100));
    }
    
    #[test]
    fn test_conflicting_votes() {
        let mut aggregator = VoteAggregator::new();
        let (_, state) = setup_test_environment();
        
        let validator_config = ValidatorConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database);
        let validator_manager = ValidatorManager::new(validator_config, state_store);
        
        // Add first vote
        let vote1 = create_test_vote(0, 1, VoteType::Prevote);
        let result1 = aggregator.add_vote(vote1, &validator_manager, &state);
        assert!(result1.is_ok());
        
        // Try to add conflicting vote from same validator
        let vote2 = create_test_vote(0, 1, VoteType::Prevote);
        let result2 = aggregator.add_vote(vote2, &validator_manager, &state);
        assert!(result2.is_err());
        
        match result2 {
            Err(FinalityError::ConflictingVotes { validator }) => {
                assert_eq!(validator, 0);
            },
            _ => panic!("Expected ConflictingVotes error"),
        }
    }
    
    #[test]
    fn test_round_advancement() {
        let (mut gadget, _) = setup_test_environment();
        
        assert_eq!(gadget.get_current_round(), 0);
        
        gadget.advance_round();
        assert_eq!(gadget.get_current_round(), 1);
        
        gadget.advance_round();
        assert_eq!(gadget.get_current_round(), 2);
    }
}
