//! State transition functions for Beam Chain consensus
//!
//! Implements block processing pipeline, epoch transitions, and state updates.

use crate::types::{BeaconState, BeaconBlock, BeaconBlockHeader, Epoch, Slot, ValidatorIndex, BlockHash};
use crate::storage::{StateStore, BlockStore, Database};
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
}

impl Default for StateTransitionConfig {
    fn default() -> Self {
        Self {
            slots_per_epoch: 32,
            seconds_per_slot: 4,  // Beam Chain 4-second slots
            genesis_time: 0,
            activation_delay: 256,
        }
    }
}

/// State transition processor
pub struct StateTransitionProcessor {
    config: StateTransitionConfig,
    state_store: StateStore,
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
        if let Some(ref _payload) = block.body.execution_payload {
            // TODO: Implement execution payload processing
        }

        Ok(())
    }

    /// Process single attestation
    fn process_attestation(
        &self,
        _state: &mut BeaconState,
        _attestation: &crate::types::Attestation,
    ) -> Result<(), StateTransitionError> {
        // TODO: Implement attestation processing
        // - Validate attestation data
        // - Check committee assignments
        // - Update validator balances
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
        // TODO: Implement justification and finalization logic
        Ok(())
    }

    /// Process rewards and penalties
    fn process_rewards_and_penalties(
        &self,
        _state: &mut BeaconState,
    ) -> Result<(), StateTransitionError> {
        // TODO: Implement reward and penalty calculations
        Ok(())
    }

    /// Process validator updates
    fn process_validator_updates(
        &self,
        _state: &mut BeaconState,
        _epoch: Epoch,
    ) -> Result<(), StateTransitionError> {
        // TODO: Implement validator activation/exit logic
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Database, DatabaseConfig, StorageConfig};

    fn setup_processor() -> StateTransitionProcessor {
        let config = StateTransitionConfig::default();
        let db_config = DatabaseConfig::default();
        let storage_config = StorageConfig::default();
        let database = Database::open(&db_config).unwrap();
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
