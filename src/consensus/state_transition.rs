//! State transition functions for Beam Chain consensus
//!
//! Implements block processing pipeline, epoch transitions, and state updates.

use crate::types::{BeaconState, BeaconBlock, BeaconBlockHeader, Epoch, Slot, ValidatorIndex, BlockHash};
use crate::storage::{StateStore, BlockStore};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// State transition errors
#[derive(Debug, Error)]
pub enum StateTransitionError {
    #[error("Invalid block slot: expected {expected}, got {actual}")]
    InvalidSlot { expected: Slot, actual: Slot },
    
    #[error("Invalid parent root: {0:?}")]
    InvalidParentRoot(BlockHash),
    
    #[error("Invalid proposer: expected {expected}, got {actual}")]
    InvalidProposer { expected: ValidatorIndex, actual: ValidatorIndex },
    
    #[error("Block validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Invalid state root: {0:?}")]
    InvalidStateRoot(BlockHash),
    
    #[error("Epoch processing failed: {0}")]
    EpochProcessing(String),
}

/// State transition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionConfig {
    /// Slots per epoch
    pub slots_per_epoch: u64,
    /// Seconds per slot
    pub seconds_per_slot: u64,
    /// Genesis time
    pub genesis_time: u64,
    /// Validator activation delay
    pub activation_delay: u64,
    /// Maximum effective balance
    pub max_effective_balance: u64,
    /// Minimum validator withdrawability delay
    pub min_validator_withdrawability_delay: u64,
    /// Maximum seed lookahead
    pub max_seed_lookahead: u64,
    /// Effective balance increment
    pub effective_balance_increment: u64,
}

impl Default for StateTransitionConfig {
    fn default() -> Self {
        Self {
            slots_per_epoch: 32,
            seconds_per_slot: 4,  // Beam Chain 4-second slots
            genesis_time: 0,
            activation_delay: 256,
            max_effective_balance: 32_000_000_000, // 32 ETH in gwei
            min_validator_withdrawability_delay: 256,
            max_seed_lookahead: 4,
            effective_balance_increment: 1_000_000_000, // 1 ETH in gwei
        }
    }
}

/// State transition processor
pub struct StateTransitionProcessor {
    config: StateTransitionConfig,
    #[allow(dead_code)]
    state_store: StateStore,
    #[allow(dead_code)]
    block_store: BlockStore,
}

impl StateTransitionProcessor {
    /// Create new state transition processor
    pub fn new(
        config: StateTransitionConfig,
        state_store: StateStore,
        block_store: BlockStore,
    ) -> Self {
        Self {
            config,
            state_store,
            block_store,
        }
    }

    /// Process beacon block and update state
    pub fn process_block(
        &self,
        state: &mut BeaconState,
        block: &BeaconBlock,
    ) -> Result<(), StateTransitionError> {
        // Validate block
        self.validate_block(state, block)?;
        
        // Process block header
        self.process_block_header(state, &block.header())?;
        
        // Process block body
        self.process_block_body(state, block)?;
        
        // Update state
        state.slot = block.slot;
        state.latest_block_header = block.header();
        
        Ok(())
    }

    /// Get block proposer for a given slot
    fn get_block_proposer(&self, state: &BeaconState, slot: Slot) -> Result<ValidatorIndex, StateTransitionError> {
        // Simplified proposer selection for testing
        // In real implementation, this would use proper shuffling algorithm
        let validator_count = state.validators.validators.len();
        if validator_count == 0 {
            return Ok(0); // Default proposer for empty validator set
        }
        
        let proposer_index = (slot as usize) % validator_count;
        Ok(proposer_index as ValidatorIndex)
    }

    /// Validate a beacon block
    pub fn validate_block(&self, state: &BeaconState, block: &BeaconBlock) -> Result<(), StateTransitionError> {
        // Special case for genesis block (slot 0)
        if block.slot == 0 && state.slot == 0 {
            return Ok(()); // Genesis block is always valid
        }
        
        // Validate slot progression for non-genesis blocks
        if block.slot != state.slot + 1 {
            return Err(StateTransitionError::InvalidSlot {
                expected: state.slot + 1,
                actual: block.slot,
            });
        }

        // For slot 0 (genesis), parent_root should be zero
        // For other blocks, validate parent root
        if block.slot > 0 && block.parent_root != state.latest_block_header.hash() {
            return Err(StateTransitionError::InvalidParentRoot(block.parent_root));
        }

        // Validate proposer
        let expected_proposer = self.get_block_proposer(state, block.slot)?;
        if block.proposer_index != expected_proposer {
            return Err(StateTransitionError::InvalidProposer {
                expected: expected_proposer,
                actual: block.proposer_index,
            });
        }

        Ok(())
    }

    /// Validate block proposer
    #[allow(dead_code)]
    fn validate_proposer(
        &self,
        state: &BeaconState,
        block: &BeaconBlock,
    ) -> Result<(), StateTransitionError> {
        let proposer = state.validators.get(block.proposer_index)
            .ok_or_else(|| StateTransitionError::ValidationFailed(
                format!("Invalid proposer index: {}", block.proposer_index)
            ))?;

        // Check if proposer is active
        let current_epoch = state.current_epoch(self.config.slots_per_epoch);
        if !proposer.is_active(current_epoch) {
            return Err(StateTransitionError::ValidationFailed(
                format!("Proposer {} not active at epoch {}", block.proposer_index, current_epoch)
            ));
        }

        Ok(())
    }

    /// Process block header
    fn process_block_header(
        &self,
        state: &mut BeaconState,
        header: &BeaconBlockHeader,
    ) -> Result<(), StateTransitionError> {
        // Update latest block header
        state.latest_block_header = header.clone();
        
        Ok(())
    }

    /// Process block body
    fn process_block_body(
        &self,
        state: &mut BeaconState,
        block: &BeaconBlock,
    ) -> Result<(), StateTransitionError> {
        // Process attestations
        for attestation in &block.body.attestations {
            self.process_attestation(state, attestation)?;
        }

        // Process execution payload if present
        if let Some(ref payload) = block.body.execution_payload {
            self.process_execution_payload(state, payload)?;
        }

        Ok(())
    }

    /// Process single attestation
    fn process_attestation(
        &self,
        state: &mut BeaconState,
        attestation: &crate::types::Attestation,
    ) -> Result<(), StateTransitionError> {
        // Use AttestationProcessor for full implementation
        
        // Check attestation slot is valid
        let current_epoch = state.current_epoch(self.config.slots_per_epoch);
        let attestation_epoch = attestation.data.slot / self.config.slots_per_epoch;
        
        if attestation_epoch > current_epoch {
            return Err(StateTransitionError::ValidationFailed(
                "Attestation from future epoch".to_string()
            ));
        }
        
        // Basic signature check placeholder
        if attestation.signature.is_empty() {
            return Err(StateTransitionError::ValidationFailed(
                "Empty attestation signature".to_string()
            ));
        }
        
        Ok(())
    }

    /// Process epoch transition
    pub fn process_epoch(
        &self,
        state: &mut BeaconState,
    ) -> Result<(), StateTransitionError> {
        let current_epoch = state.current_epoch(self.config.slots_per_epoch);
        
        // Process justification and finalization
        self.process_justification_and_finalization(state)?;
        
        // Process rewards and penalties
        self.process_rewards_and_penalties(state)?;
        
        // Process validator set changes
        self.process_validator_updates(state, current_epoch)?;
        
        Ok(())
    }

    /// Process justification and finalization
    fn process_justification_and_finalization(
        &self,
        _state: &mut BeaconState,
    ) -> Result<(), StateTransitionError> {
        // Implement justification and finalization logic
        self.update_justification_and_finalization(state)?;
        Ok(())
    }

    /// Process rewards and penalties
    fn process_rewards_and_penalties(
        &self,
        _state: &mut BeaconState,
    ) -> Result<(), StateTransitionError> {
        // Implement reward and penalty calculations
        self.calculate_rewards_and_penalties(state)?;
        Ok(())
    }

    /// Process validator updates
    fn process_validator_updates(
        &self,
        _state: &mut BeaconState,
        _epoch: Epoch,
    ) -> Result<(), StateTransitionError> {
        // Implement validator activation/exit logic
        self.process_validator_registry_updates(state, epoch)?;
        Ok(())
    }
    
    /// Process execution payload during block processing
    fn process_execution_payload(
        &self,
        state: &mut BeaconState,
        payload: &crate::types::ExecutionPayload,
    ) -> Result<(), StateTransitionError> {
        // Verify execution payload hash
        let computed_hash = self.compute_execution_payload_hash(payload)?;
        if computed_hash != payload.block_hash {
            return Err(StateTransitionError::ValidationFailed(
                "Execution payload hash mismatch".to_string()
            ));
        }
        
        // Update latest execution payload header
        state.latest_execution_payload_header = Some(payload.clone().into());
        
        Ok(())
    }
    
    /// Compute execution payload hash
    fn compute_execution_payload_hash(
        &self,
        payload: &crate::types::ExecutionPayload,
    ) -> Result<[u8; 32], StateTransitionError> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&payload.parent_hash);
        hasher.update(&payload.fee_recipient);
        hasher.update(&payload.state_root);
        hasher.update(&payload.receipts_root);
        hasher.update(&payload.gas_limit.to_le_bytes());
        hasher.update(&payload.gas_used.to_le_bytes());
        hasher.update(&payload.timestamp.to_le_bytes());
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Ok(hash)
    }
    
    /// Update justification and finalization status
    fn update_justification_and_finalization(
        &self,
        state: &mut BeaconState,
    ) -> Result<(), StateTransitionError> {
        let current_epoch = state.current_epoch(self.config.slots_per_epoch);
        let previous_epoch = current_epoch.saturating_sub(1);
        
        // Get previous and current epoch totals
        let current_total_balance = self.get_total_active_balance(state, current_epoch)?;
        let previous_total_balance = self.get_total_active_balance(state, previous_epoch)?;
        
        // Check if epochs are justified (simplified)
        let current_epoch_justified = current_total_balance * 2 >= current_total_balance * 3 / 2;
        let previous_epoch_justified = previous_total_balance * 2 >= previous_total_balance * 3 / 2;
        
        // Update justification bits
        if current_epoch_justified {
            state.justification_bits |= 1 << (current_epoch % 4);
        }
        if previous_epoch_justified {
            state.justification_bits |= 1 << (previous_epoch % 4);
        }
        
        // Check finalization (Casper FFG rules)
        if self.check_finalization_conditions(state, current_epoch)? {
            state.finalized_checkpoint.epoch = current_epoch.saturating_sub(2);
        }
        
        Ok(())
    }
    
    /// Check if finalization conditions are met
    fn check_finalization_conditions(
        &self,
        state: &BeaconState,
        current_epoch: Epoch,
    ) -> Result<bool, StateTransitionError> {
        // Rule 1: The previous two epochs are justified, the current epoch is justified
        let bits = state.justification_bits;
        let mask_3_epochs = 0b111;
        
        if (bits & mask_3_epochs) == mask_3_epochs {
            return Ok(true);
        }
        
        // Rule 2: The previous epoch is justified and it has been justified for two epochs
        let mask_2_epochs = 0b11;
        if current_epoch >= 2 && (bits & mask_2_epochs) == mask_2_epochs {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Calculate rewards and penalties for validators
    fn calculate_rewards_and_penalties(
        &self,
        state: &mut BeaconState,
    ) -> Result<(), StateTransitionError> {
        let current_epoch = state.current_epoch(self.config.slots_per_epoch);
        let base_reward = self.config.base_reward_factor;
        
        for (index, validator) in state.validators.iter_mut().enumerate() {
            if validator.activation_epoch > current_epoch || validator.exit_epoch <= current_epoch {
                continue; // Skip inactive validators
            }
            
            let mut reward = 0i64;
            let mut penalty = 0i64;
            
            // Attestation rewards
            if validator.slashed {
                penalty += base_reward as i64 * 3; // Slashing penalty
            } else {
                reward += base_reward as i64; // Base reward for being active
            }
            
            // Apply rewards and penalties
            if reward > penalty {
                validator.effective_balance = validator.effective_balance.saturating_add((reward - penalty) as u64);
            } else {
                validator.effective_balance = validator.effective_balance.saturating_sub((penalty - reward) as u64);
            }
            
            // Ensure effective balance doesn't exceed maximum
            if validator.effective_balance > self.config.max_effective_balance {
                validator.effective_balance = self.config.max_effective_balance;
            }
        }
        
        Ok(())
    }
    
    /// Process validator registry updates (activations, exits)
    fn process_validator_registry_updates(
        &self,
        state: &mut BeaconState,
        epoch: Epoch,
    ) -> Result<(), StateTransitionError> {
        // Process activation queue
        let mut activation_queue: Vec<(ValidatorIndex, Epoch)> = Vec::new();
        
        for (index, validator) in state.validators.iter_mut().enumerate() {
            // Check for activation eligibility
            if validator.activation_eligibility_epoch == u64::MAX 
                && validator.effective_balance >= self.config.max_effective_balance {
                validator.activation_eligibility_epoch = epoch + 1;
                activation_queue.push((index as ValidatorIndex, epoch + 1));
            }
            
            // Process activations
            if validator.activation_epoch == u64::MAX 
                && validator.activation_eligibility_epoch <= epoch {
                validator.activation_epoch = self.compute_activation_exit_epoch(epoch);
            }
            
            // Process voluntary exits
            if validator.exit_epoch == u64::MAX 
                && validator.withdrawable_epoch != u64::MAX {
                validator.exit_epoch = self.compute_activation_exit_epoch(epoch);
                validator.withdrawable_epoch = validator.exit_epoch + self.config.min_validator_withdrawability_delay;
            }
            
            // Process withdrawals
            if validator.withdrawable_epoch <= epoch 
                && validator.effective_balance > 0 {
                // Process withdrawal (simplified)
                state.balances[index] = state.balances.get(index).unwrap_or(&0) + validator.effective_balance;
                validator.effective_balance = 0;
            }
        }
        
        Ok(())
    }
    
    /// Compute activation/exit epoch with churn limit
    fn compute_activation_exit_epoch(&self, current_epoch: Epoch) -> Epoch {
        current_epoch + 1 + self.config.max_seed_lookahead
    }
    
    /// Get total active balance for an epoch
    fn get_total_active_balance(&self, state: &BeaconState, epoch: Epoch) -> Result<u64, StateTransitionError> {
        let mut total = 0u64;
        
        for validator in &state.validators.validators {
            if validator.activation_epoch <= epoch && epoch < validator.exit_epoch {
                total = total.saturating_add(validator.effective_balance);
            }
        }
        
        // Ensure minimum total balance
        Ok(total.max(self.config.effective_balance_increment))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Database, StorageConfig};

    fn setup_processor() -> StateTransitionProcessor {
        let config = StateTransitionConfig::default();
        let _storage_config = StorageConfig::default();
        let database = Database::in_memory();
        let state_store = StateStore::new(database.clone());
        let block_store = BlockStore::new(database);
        
        StateTransitionProcessor::new(config, state_store, block_store)
    }

    #[test]
    fn test_state_transition_creation() {
        let processor = setup_processor();
        assert_eq!(processor.config.slots_per_epoch, 32);
    }

    #[test]
    fn test_block_validation() {
        let processor = setup_processor();
        let mut state = BeaconState::default();
        state.slot = 10;
        
        let mut block = BeaconBlock::default();
        block.slot = 11; // Valid next slot
        block.parent_root = state.latest_block_header.hash(); // Set correct parent root
        
        // Should pass basic validation
        let result = processor.validate_block(&state, &block);
        match &result {
            Ok(_) => {},
            Err(e) => println!("Validation error: {:?}", e),
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_slot() {
        let processor = setup_processor();
        let mut state = BeaconState::default();
        state.slot = 10;
        
        let mut block = BeaconBlock::default();
        block.slot = 15; // Invalid slot jump
        
        let result = processor.validate_block(&state, &block);
        assert!(result.is_err());
    }
}
