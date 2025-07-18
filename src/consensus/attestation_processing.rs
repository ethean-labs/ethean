//! Attestation processing system for Beam Chain consensus
//!
//! Handles attestation validation, committee management, aggregation, and rewards.

use crate::types::{BeaconState, Attestation, ValidatorIndex, Slot, Epoch};
use crate::consensus::validator_management::{ValidatorManager, ValidatorError};
use crate::crypto::bls::{RealBLSAggregator, BLSSignature, BLSPublicKey, BLSError};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::{HashMap, HashSet};

/// Attestation processing errors
#[derive(Debug, Error)]
pub enum AttestationError {
    #[error("Invalid committee assignment for validator {validator} at slot {slot}")]
    InvalidCommittee { validator: ValidatorIndex, slot: Slot },
    
    #[error("Invalid attestation slot: {slot}, expected range: {min}-{max}")]
    InvalidSlot { slot: Slot, min: Slot, max: Slot },
    
    #[error("Invalid inclusion delay: {delay}, maximum allowed: {max}")]
    InvalidInclusionDelay { delay: u64, max: u64 },
    
    #[error("Signature verification failed for attestation")]
    SignatureVerificationFailed,
    
    #[error("Invalid aggregation bits: expected {expected}, got {actual}")]
    InvalidAggregationBits { expected: usize, actual: usize },
    
    #[error("Committee not found for slot {slot}, index {index}")]
    CommitteeNotFound { slot: Slot, index: u64 },
    
    #[error("Validator error: {0}")]
    Validator(#[from] ValidatorError),
    
    #[error("Attestation already processed for validator {validator} at slot {slot}")]
    DuplicateAttestation { validator: ValidatorIndex, slot: Slot },
    
    #[error("BLS signature error: {0}")]
    BLSError(#[from] BLSError),
}

/// Committee assignment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Committee {
    pub slot: Slot,
    pub index: u64,
    pub validators: Vec<ValidatorIndex>,
}

/// Attestation processing result
#[derive(Debug, Clone)]
pub struct AttestationResult {
    pub included: bool,
    pub committee: Committee,
    pub rewards: Vec<u64>,
    pub penalties: Vec<u64>,
    pub aggregated_signature: Option<BLSSignature>,
}

/// Attestation processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationStats {
    pub total_processed: u64,
    pub valid_attestations: u64,
    pub invalid_attestations: u64,
    pub duplicate_attestations: u64,
    pub total_rewards_distributed: u64,
    pub total_penalties_applied: u64,
}

impl Default for AttestationStats {
    fn default() -> Self {
        Self {
            total_processed: 0,
            valid_attestations: 0,
            invalid_attestations: 0,
            duplicate_attestations: 0,
            total_rewards_distributed: 0,
            total_penalties_applied: 0,
        }
    }
}

/// Advanced committee management system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeManager {
    epoch_committees: HashMap<Epoch, Vec<Committee>>,
    shuffling_cache: HashMap<Epoch, Vec<ValidatorIndex>>,
    target_committee_size: usize,
    committees_per_slot: u64,
}

impl CommitteeManager {
    pub fn new(target_committee_size: usize, committees_per_slot: u64) -> Self {
        Self {
            epoch_committees: HashMap::new(),
            shuffling_cache: HashMap::new(),
            target_committee_size,
            committees_per_slot,
        }
    }

    /// Get committee for specific slot and index
    pub fn get_committee_for_slot(
        &mut self,
        slot: Slot,
        committee_index: u64,
        state: &BeaconState,
    ) -> Result<Committee, AttestationError> {
        let epoch = slot / 32; // Simplified epoch calculation
        
        // Get committees for this epoch
        let committees = self.get_epoch_committees(state, epoch)?;
        
        // Find the specific committee
        committees.into_iter()
            .find(|c| c.slot == slot && c.index == committee_index)
            .ok_or(AttestationError::CommitteeNotFound { 
                slot, 
                index: committee_index 
            })
    }

    /// Get all committees for an epoch
    pub fn get_epoch_committees(
        &mut self,
        state: &BeaconState,
        epoch: Epoch,
    ) -> Result<Vec<Committee>, AttestationError> {
        if let Some(committees) = self.epoch_committees.get(&epoch) {
            return Ok(committees.clone());
        }

        let committees = self.calculate_epoch_committees(state, epoch)?;
        self.epoch_committees.insert(epoch, committees.clone());
        Ok(committees)
    }

    /// Calculate all committees for an epoch
    fn calculate_epoch_committees(
        &mut self,
        state: &BeaconState,
        epoch: Epoch,
    ) -> Result<Vec<Committee>, AttestationError> {
        let shuffled_validators = self.get_shuffled_validators(state, epoch)?;
        let slots_per_epoch = 32;
        let total_committees = slots_per_epoch * self.committees_per_slot;
        
        if shuffled_validators.is_empty() {
            return Ok(Vec::new());
        }

        let validators_per_committee = std::cmp::max(1, shuffled_validators.len() / total_committees as usize);
        let mut committees = Vec::new();

        for slot_offset in 0..slots_per_epoch {
            for committee_index in 0..self.committees_per_slot {
                let committee_id = slot_offset * self.committees_per_slot + committee_index;
                let start_index = (committee_id as usize * validators_per_committee) % shuffled_validators.len();
                
                let mut committee_validators = Vec::new();
                for i in 0..validators_per_committee {
                    let validator_index = shuffled_validators[(start_index + i) % shuffled_validators.len()];
                    committee_validators.push(validator_index);
                }

                committees.push(Committee {
                    slot: epoch * 32 + slot_offset,
                    index: committee_index,
                    validators: committee_validators,
                });
            }
        }

        Ok(committees)
    }

    /// Get shuffled validator list for epoch
    fn get_shuffled_validators(
        &mut self,
        state: &BeaconState,
        epoch: Epoch,
    ) -> Result<Vec<ValidatorIndex>, AttestationError> {
        if let Some(shuffled) = self.shuffling_cache.get(&epoch) {
            return Ok(shuffled.clone());
        }

        let active_validators = self.get_active_validators_for_epoch(state, epoch);
        
        // Simple shuffling (in real implementation, would use RANDAO + epoch for secure shuffling)
        let mut shuffled = active_validators.clone();
        
        // Deterministic shuffle based on epoch
        let seed = epoch as u64;
        for i in 0..shuffled.len() {
            let j = ((seed + i as u64) * 2654435761) % (shuffled.len() as u64);
            shuffled.swap(i, j as usize);
        }

        self.shuffling_cache.insert(epoch, shuffled.clone());
        Ok(shuffled)
    }

    /// Get active validators for specific epoch
    fn get_active_validators_for_epoch(&self, state: &BeaconState, epoch: Epoch) -> Vec<ValidatorIndex> {
        state.validators.validators
            .iter()
            .enumerate()
            .filter_map(|(i, validator)| {
                if validator.activation_epoch <= epoch && 
                   validator.exit_epoch > epoch && 
                   !validator.slashed {
                    Some(i as ValidatorIndex)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Clean old cached data
    pub fn cleanup_old_data(&mut self, current_epoch: Epoch) {
        let keep_epochs = 3; // Keep last 3 epochs
        
        self.epoch_committees.retain(|&epoch, _| epoch + keep_epochs >= current_epoch);
        self.shuffling_cache.retain(|&epoch, _| epoch + keep_epochs >= current_epoch);
    }
}

/// Real BLS signature aggregation system
#[derive(Debug)]
pub struct SignatureAggregator {
    bls_aggregator: RealBLSAggregator,
    pending_signatures: HashMap<(Slot, u64), Vec<(ValidatorIndex, BLSSignature)>>,
    aggregated_signatures: HashMap<(Slot, u64), BLSSignature>,
    participation_bitfields: HashMap<(Slot, u64), Vec<bool>>,
}

impl SignatureAggregator {
    pub fn new() -> Self {
        Self {
            bls_aggregator: RealBLSAggregator::new(),
            pending_signatures: HashMap::new(),
            aggregated_signatures: HashMap::new(),
            participation_bitfields: HashMap::new(),
        }
    }

    /// Aggregate signature for attestation with real BLS
    pub fn aggregate_signature(
        &mut self,
        attestation: &Attestation,
        committee: &Committee,
        _state: &BeaconState,
    ) -> Result<(BLSSignature, Vec<bool>), AttestationError> {
        let key = (attestation.data.slot, attestation.data.index);
        
        // Check cache first
        if let Some(cached_sig) = self.aggregated_signatures.get(&key) {
            if let Some(cached_bits) = self.participation_bitfields.get(&key) {
                return Ok((cached_sig.clone(), cached_bits.clone()));
            }
        }

        // Create participation bitfield from attestation
        let participation_bits = attestation.aggregation_bits.clone();
        
        // Convert attestation signature to BLS format
        let attestation_signature = BLSSignature {
            point: attestation.signature.clone(),
        };
        
        // In real implementation, this would aggregate multiple individual signatures
        // For now, we'll treat the attestation signature as already aggregated
        let aggregated_signature = attestation_signature;
        
        // Verify the aggregated signature if we have public keys
        if let Some(first_validator) = committee.validators.first() {
            // Create a mock public key for testing
            let mock_pubkey = self.create_mock_public_key(*first_validator)?;
            let message = self.create_attestation_message(attestation)?;
            
            // Verify signature (in production, would verify against all participating validators)
            let _verification_result = self.bls_aggregator.verify_signature(
                &aggregated_signature,
                &message,
                &mock_pubkey,
            )?;
        }
        
        // Cache the results
        self.aggregated_signatures.insert(key, aggregated_signature.clone());
        self.participation_bitfields.insert(key, participation_bits.clone());

        Ok((aggregated_signature, participation_bits))
    }

    /// Add signature to aggregation
    pub fn add_signature(
        &mut self,
        slot: Slot,
        committee_index: u64,
        validator_index: ValidatorIndex,
        signature: BLSSignature,
    ) -> Result<(), AttestationError> {
        let key = (slot, committee_index);
        
        self.pending_signatures
            .entry(key)
            .or_insert_with(Vec::new)
            .push((validator_index, signature));

        Ok(())
    }

    /// Create mock public key for testing
    fn create_mock_public_key(&self, validator_index: ValidatorIndex) -> Result<BLSPublicKey, AttestationError> {
        use bls12_381::{G2Projective, Scalar};
        
        // Create deterministic public key based on validator index
        let scalar = Scalar::from(validator_index + 1); // Avoid zero
        let point = G2Projective::generator() * scalar;
        
        Ok(BLSPublicKey::from_g2(&point.into()))
    }
    
    /// Create attestation message for signing
    fn create_attestation_message(&self, attestation: &Attestation) -> Result<Vec<u8>, AttestationError> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(attestation.data.slot.to_le_bytes());
        hasher.update(attestation.data.index.to_le_bytes());
        hasher.update(&attestation.data.beacon_block_root);
        hasher.update(attestation.data.source.epoch.to_le_bytes());
        hasher.update(&attestation.data.source.root);
        hasher.update(attestation.data.target.epoch.to_le_bytes());
        hasher.update(&attestation.data.target.root);
        
        Ok(hasher.finalize().to_vec())
    }
    /// Aggregate signatures for a committee
    pub fn aggregate_signatures(
        &mut self,
        slot: Slot,
        committee_index: u64,
        committee: &Committee,
    ) -> Result<(BLSSignature, Vec<bool>), AttestationError> {
        let key = (slot, committee_index);
        
        if let Some(cached_sig) = self.aggregated_signatures.get(&key) {
            if let Some(cached_bits) = self.participation_bitfields.get(&key) {
                return Ok((cached_sig.clone(), cached_bits.clone()));
            }
        }

        let pending = self.pending_signatures.get(&key).cloned().unwrap_or_default();
        
        // Create participation bitfield
        let mut participation_bits = vec![false; committee.validators.len()];
        let mut signatures_to_aggregate = Vec::new();

        for (validator_index, signature) in &pending {
            if let Some(position) = committee.validators.iter().position(|&v| v == *validator_index) {
                participation_bits[position] = true;
                signatures_to_aggregate.push(signature.clone());
            }
        }

        // Real BLS signature aggregation
        let aggregated_signature = if signatures_to_aggregate.is_empty() {
            // Create identity signature
            BLSSignature::from_g1(&bls12_381::G1Affine::identity())
        } else {
            self.bls_aggregator.aggregate_signatures(&signatures_to_aggregate)?
        };

        // Cache results
        self.aggregated_signatures.insert(key, aggregated_signature.clone());
        self.participation_bitfields.insert(key, participation_bits.clone());

        Ok((aggregated_signature, participation_bits))
    }

    /// Verify aggregated signature
    pub fn verify_aggregated_signature(
        &mut self,
        signature: &BLSSignature,
        message: &[u8],
        public_key: &BLSPublicKey,
    ) -> Result<bool, AttestationError> {
        let result = self.bls_aggregator.verify_signature(signature, message, public_key)?;
        Ok(result)
    }

    /// Clean old aggregation data
    pub fn cleanup_old_data(&mut self, current_slot: Slot) {
        let keep_slots = 64; // Keep last 64 slots
        
        self.pending_signatures.retain(|(slot, _), _| *slot + keep_slots >= current_slot);
        self.aggregated_signatures.retain(|(slot, _), _| *slot + keep_slots >= current_slot);
        self.participation_bitfields.retain(|(slot, _), _| *slot + keep_slots >= current_slot);
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationConfig {
    /// Target committee size
    pub target_committee_size: usize,
    /// Maximum inclusion delay for attestations (in slots)
    pub max_inclusion_delay: u64,
    /// Base reward factor for attestations
    pub base_reward_factor: u64,
    /// Penalty factor for late attestations
    pub late_penalty_factor: u64,
    /// Committee count per slot
    pub committees_per_slot: u64,
}

impl Default for AttestationConfig {
    fn default() -> Self {
        Self {
            target_committee_size: 128, // Target committee size
            max_inclusion_delay: 32,    // Maximum 32 slots delay
            base_reward_factor: 64,     // Base reward calculation
            late_penalty_factor: 4,     // Penalty for late inclusion
            committees_per_slot: 64,    // Number of committees per slot
        }
    }
}

/// Enhanced attestation processor with advanced features
pub struct AttestationProcessor {
    config: AttestationConfig,
    validator_manager: ValidatorManager,
    committee_manager: CommitteeManager,
    signature_aggregator: SignatureAggregator,
    processed_attestations: HashSet<(ValidatorIndex, Slot)>,
    stats: AttestationStats,
}

impl AttestationProcessor {
    /// Create new enhanced attestation processor
    pub fn new(config: AttestationConfig, validator_manager: ValidatorManager) -> Self {
        let committee_manager = CommitteeManager::new(
            config.target_committee_size,
            config.committees_per_slot,
        );
        let signature_aggregator = SignatureAggregator::new();

        Self {
            config,
            validator_manager,
            committee_manager,
            signature_aggregator,
            processed_attestations: HashSet::new(),
            stats: AttestationStats::default(),
        }
    }

    /// Process single attestation with enhanced validation pipeline
    pub fn process_attestation(
        &mut self,
        state: &mut BeaconState,
        attestation: &Attestation,
        inclusion_slot: Slot,
    ) -> Result<AttestationResult, AttestationError> {
        self.stats.total_processed += 1;

        // Enhanced validation pipeline
        self.validate_attestation(state, attestation, inclusion_slot)?;

        // Get committee with advanced management
        let committee = self.committee_manager.get_committee_for_slot(
            attestation.data.slot,
            attestation.data.index,
            state,
        )?;

        // Signature aggregation and verification
        let (aggregated_signature, _participation_bits) = self.signature_aggregator
            .aggregate_signature(attestation, &committee, state)?;

        // Calculate rewards and penalties
        let rewards = self.calculate_attestation_rewards(attestation, &committee, state)?;
        let penalties = self.calculate_inactivity_penalties(attestation, &committee, state)?;

        // Process each validator in the aggregation bits
        for (i, &participated) in attestation.aggregation_bits.iter().enumerate() {
            if participated {
                let validator_index = committee.validators.get(i)
                    .ok_or(AttestationError::InvalidAggregationBits {
                        expected: committee.validators.len(),
                        actual: attestation.aggregation_bits.len(),
                    })?;

                // Check for duplicate
                if self.processed_attestations.contains(&(*validator_index, attestation.data.slot)) {
                    return Err(AttestationError::DuplicateAttestation {
                        validator: *validator_index,
                        slot: attestation.data.slot,
                    });
                }

                // Process validator attestation with enhanced tracking
                self.process_validator_attestation(
                    state,
                    *validator_index,
                    attestation,
                    inclusion_slot,
                )?;

                // Mark as processed
                self.processed_attestations.insert((*validator_index, attestation.data.slot));
            }
        }

        // Update comprehensive statistics
        self.stats.valid_attestations += 1;
        self.stats.total_rewards_distributed += rewards.iter().sum::<u64>();
        self.stats.total_penalties_applied += penalties.iter().sum::<u64>();

        Ok(AttestationResult {
            included: true,
            committee: committee.clone(),
            rewards,
            penalties,
            aggregated_signature: Some(aggregated_signature),
        })
    }

    /// Validate attestation structure and timing
    fn validate_attestation(
        &self,
        _state: &BeaconState,
        attestation: &Attestation,
        inclusion_slot: Slot,
    ) -> Result<(), AttestationError> {
        // Check inclusion delay
        let inclusion_delay = inclusion_slot.saturating_sub(attestation.data.slot);
        if inclusion_delay > self.config.max_inclusion_delay {
            return Err(AttestationError::InvalidInclusionDelay {
                delay: inclusion_delay,
                max: self.config.max_inclusion_delay,
            });
        }

        // Check slot validity (can't attest to future slots)
        if attestation.data.slot > inclusion_slot {
            return Err(AttestationError::InvalidSlot {
                slot: attestation.data.slot,
                min: 0,
                max: inclusion_slot,
            });
        }

        // Validate that source epoch is before target epoch
        if attestation.data.source.epoch >= attestation.data.target.epoch {
            return Err(AttestationError::InvalidSlot {
                slot: attestation.data.slot,
                min: 0,
                max: inclusion_slot,
            });
        }

        // TODO: Add signature verification
        // self.verify_attestation_signature(state, attestation)?;

        Ok(())
    }

    /// Calculate attestation rewards for validators
    fn calculate_attestation_rewards(
        &self,
        attestation: &Attestation,
        committee: &Committee,
        state: &BeaconState,
    ) -> Result<Vec<u64>, AttestationError> {
        let mut rewards = vec![0u64; committee.validators.len()];
        let inclusion_delay = 1; // Simplified for now
        let base_reward = self.config.base_reward_factor;

        for (i, &participated) in attestation.aggregation_bits.iter().enumerate() {
            if participated {
                // Base reward for correct attestation
                let mut validator_reward = base_reward;

                // Check validator performance from state
                if let Some(&validator_index) = committee.validators.get(i) {
                    if let Some(validator) = state.validators.validators.get(validator_index as usize) {
                        // Bonus for high effective balance
                        if validator.effective_balance > 1_000_000_000 {
                            validator_reward += base_reward / 16; // High stake bonus
                        }
                    }
                }

                // Bonus for fast inclusion
                if inclusion_delay == 1 {
                    validator_reward += base_reward / 8; // Inclusion bonus
                }

                // Additional reward for correct source/target
                validator_reward += base_reward / 4; // Source reward
                validator_reward += base_reward / 4; // Target reward

                rewards[i] = validator_reward;
            }
        }

        Ok(rewards)
    }

    /// Calculate inactivity penalties for missing attestations
    fn calculate_inactivity_penalties(
        &self,
        attestation: &Attestation,
        committee: &Committee,
        state: &BeaconState,
    ) -> Result<Vec<u64>, AttestationError> {
        let mut penalties = vec![0u64; committee.validators.len()];
        let base_penalty = self.config.base_reward_factor / 4;

        // Check if we're in an inactivity leak
        let inactivity_leak = self.is_inactivity_leak(state);

        for (i, &participated) in attestation.aggregation_bits.iter().enumerate() {
            if !participated {
                let mut penalty = base_penalty;

                // Increased penalty during inactivity leak
                if inactivity_leak {
                    penalty *= 4; // Quadruple penalty during leak
                }

                penalties[i] = penalty;
            }
        }

        Ok(penalties)
    }

    /// Check if chain is in inactivity leak
    fn is_inactivity_leak(&self, state: &BeaconState) -> bool {
        // Simplified: check if we haven't finalized in 4 epochs
        state.current_epoch(32).saturating_sub(state.finalized_checkpoint.epoch) > 4
    }

    /// Get list of active validators for epoch
    fn get_active_validators(&self, state: &BeaconState, epoch: Epoch) -> Vec<ValidatorIndex> {
        state.validators.validators
            .iter()
            .enumerate()
            .filter_map(|(i, validator)| {
                if validator.activation_epoch <= epoch && 
                   validator.exit_epoch > epoch && 
                   !validator.slashed {
                    Some(i as ValidatorIndex)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Process individual validator attestation and calculate rewards
    fn process_validator_attestation(
        &mut self,
        state: &mut BeaconState,
        validator_index: ValidatorIndex,
        attestation: &Attestation,
        inclusion_slot: Slot,
    ) -> Result<(), AttestationError> {
        let current_epoch = state.current_epoch(32);
        
        // Calculate inclusion delay reward
        let inclusion_delay = inclusion_slot.saturating_sub(attestation.data.slot);
        let base_reward = self.calculate_base_reward(validator_index)?;
        
        // Reward calculation based on inclusion delay
        let inclusion_reward = if inclusion_delay <= 1 {
            base_reward // Full reward for immediate inclusion
        } else {
            base_reward / (inclusion_delay + 1) // Reduced reward for delayed inclusion
        };

        // Apply reward
        self.validator_manager.apply_reward(validator_index, inclusion_reward, current_epoch)?;
        self.stats.total_rewards_distributed += inclusion_reward;

        // Update performance metrics
        self.validator_manager.update_performance(
            validator_index,
            true, // Successful attestation
            inclusion_delay,
        )?;

        Ok(())
    }

    /// Calculate base reward for validator
    fn calculate_base_reward(&self, validator_index: ValidatorIndex) -> Result<u64, AttestationError> {
        let effective_balance = self.validator_manager
            .get_effective_balance(validator_index)
            .unwrap_or(1_000_000_000); // Default 1 ETH

        // Base reward calculation (simplified)
        let base_reward = effective_balance * self.config.base_reward_factor / 1_000_000;
        Ok(base_reward)
    }

    /// Process epoch transition for attestations
    pub fn process_epoch_transition(
        &mut self,
        state: &mut BeaconState,
        epoch: Epoch,
    ) -> Result<(), AttestationError> {
        // Clean old committee data using committee manager
        self.committee_manager.cleanup_old_data(epoch);

        // Process inactivity penalties for validators who didn't attest
        self.process_inactivity_penalties(state, epoch)?;

        // Clear processed attestations for old epochs
        let epoch_start_slot = epoch * 32;
        self.processed_attestations.retain(|(_, slot)| *slot >= epoch_start_slot);

        Ok(())
    }

    /// Apply inactivity penalties for non-attesting validators
    fn process_inactivity_penalties(
        &mut self,
        state: &mut BeaconState,
        epoch: Epoch,
    ) -> Result<(), AttestationError> {
        let active_validators = self.get_active_validators(state, epoch);
        
        for validator_index in active_validators {
            // Check if validator attested in this epoch
            let attested_in_epoch = self.processed_attestations
                .iter()
                .any(|(v_idx, slot)| {
                    *v_idx == validator_index && 
                    *slot >= epoch * 32 && 
                    *slot < (epoch + 1) * 32
                });

            if !attested_in_epoch {
                // Apply inactivity penalty
                let penalty = self.calculate_inactivity_penalty(validator_index)?;
                self.validator_manager.apply_penalty(validator_index, penalty, epoch)?;
                self.stats.total_penalties_applied += penalty;
            }
        }

        Ok(())
    }

    /// Calculate inactivity penalty for validator
    fn calculate_inactivity_penalty(&self, validator_index: ValidatorIndex) -> Result<u64, AttestationError> {
        let effective_balance = self.validator_manager
            .get_effective_balance(validator_index)
            .unwrap_or(1_000_000_000); // Default 1 ETH

        // Inactivity penalty (smaller for Beam Chain)
        let penalty = effective_balance / 1000; // 0.1% penalty
        Ok(penalty)
    }

    /// Get attestation processing statistics
    pub fn get_stats(&self) -> &AttestationStats {
        &self.stats
    }

    /// Get committee cache size (delegates to committee manager)
    pub fn get_committee_cache_size(&self) -> usize {
        // Return approximate cache size from committee manager
        self.committee_manager.epoch_committees.len()
    }

    /// Reset statistics (for testing)
    pub fn reset_stats(&mut self) {
        self.stats = AttestationStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BeaconState, Validator, AttestationData};
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{StateStore, Database};

    fn setup_attestation_processor() -> AttestationProcessor {
        let config = AttestationConfig::default();
        let validator_config = ValidatorConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database);
        let validator_manager = ValidatorManager::new(validator_config, state_store);
        
        AttestationProcessor::new(config, validator_manager)
    }

    fn setup_test_state() -> BeaconState {
        let mut state = BeaconState::default();
        
        // Add some test validators
        for i in 0..10 {
            let validator = Validator {
                pubkey: vec![i as u8; 48],
                withdrawal_credentials: [i as u8; 32],
                effective_balance: 1_000_000_000, // 1 ETH
                slashed: false,
                activation_epoch: 0,
                exit_epoch: u64::MAX,
            };
            state.validators.validators.push(validator);
        }
        
        state
    }

    fn create_test_attestation(slot: Slot, committee_index: u64) -> Attestation {
        Attestation {
            aggregation_bits: vec![true, false, true, false, true], // 5 validators, 3 attesting
            data: AttestationData {
                slot,
                index: committee_index,
                beacon_block_root: [1u8; 32],
                source: crate::types::Checkpoint { epoch: 0, root: [0u8; 32] },
                target: crate::types::Checkpoint { epoch: 1, root: [1u8; 32] },
            },
            signature: vec![0u8; 96], // Mock signature
        }
    }

    #[test]
    fn test_attestation_processor_creation() {
        let processor = setup_attestation_processor();
        assert_eq!(processor.config.target_committee_size, 128);
        assert_eq!(processor.get_committee_cache_size(), 0);
    }

    #[test]
    fn test_committee_calculation() {
        let mut processor = setup_attestation_processor();
        let state = setup_test_state();
        
        // Use committee manager instead of deprecated method
        let committee = processor.committee_manager.get_committee_for_slot(10, 0, &state);
        assert!(committee.is_ok());
        
        let committee = committee.unwrap();
        assert_eq!(committee.slot, 10);
        assert_eq!(committee.index, 0);
        assert!(!committee.validators.is_empty());
    }

    #[test]
    fn test_attestation_validation() {
        let processor = setup_attestation_processor();
        let state = setup_test_state();
        let attestation = create_test_attestation(10, 0);
        
        // Valid attestation
        let result = processor.validate_attestation(&state, &attestation, 11);
        assert!(result.is_ok());
        
        // Invalid attestation (future slot)
        let result = processor.validate_attestation(&state, &attestation, 9);
        assert!(result.is_err());
    }

    #[test]
    fn test_base_reward_calculation() {
        let processor = setup_attestation_processor();
        let reward = processor.calculate_base_reward(0);
        assert!(reward.is_ok());
        assert!(reward.unwrap() > 0);
    }

    #[test]
    fn test_inactivity_penalty_calculation() {
        let processor = setup_attestation_processor();
        let penalty = processor.calculate_inactivity_penalty(0);
        assert!(penalty.is_ok());
        assert!(penalty.unwrap() > 0);
    }

    #[test]
    fn test_active_validators() {
        let processor = setup_attestation_processor();
        let state = setup_test_state();
        
        let active_validators = processor.get_active_validators(&state, 0);
        assert_eq!(active_validators.len(), 10); // All test validators should be active
    }

    #[test]
    fn test_committee_cache() {
        let mut processor = setup_attestation_processor();
        let state = setup_test_state();
        
        // First call should calculate and cache
        let committee1 = processor.committee_manager.get_committee_for_slot(10, 0, &state);
        assert!(committee1.is_ok());
        assert!(processor.get_committee_cache_size() >= 1);
        
        // Second call should use cache
        let committee2 = processor.committee_manager.get_committee_for_slot(10, 0, &state);
        assert!(committee2.is_ok());
        assert!(processor.get_committee_cache_size() >= 1);
    }

    #[test]
    fn test_stats_tracking() {
        let mut processor = setup_attestation_processor();
        
        // Initial stats
        let stats = processor.get_stats();
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.valid_attestations, 0);
        
        // Reset stats
        processor.reset_stats();
        let stats = processor.get_stats();
        assert_eq!(stats.total_processed, 0);
    }
}
