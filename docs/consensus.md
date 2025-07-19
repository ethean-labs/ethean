# Consensus Implementation

## Overview

Panro implements the Ethereum Beacon Chain consensus mechanism based on proof-of-stake with LMD-GHOST fork choice rule. This document details the consensus implementation architecture and components.

## Consensus Components

### 1. State Transition Function

The state transition function processes blocks and applies state changes according to the Beacon Chain specification.

```rust
pub struct StateProcessor {
    config: ChainConfig,
    validator_registry: ValidatorRegistry,
    fork_choice: ForkChoice,
}

impl StateProcessor {
    pub async fn process_block(
        &self,
        state: &mut BeaconState,
        block: &BeaconBlock,
    ) -> Result<(), ConsensusError> {
        // Validate block structure
        self.validate_block_structure(block).await?;
        
        // Process block header
        self.process_block_header(state, &block.header).await?;
        
        // Process RANDAO
        self.process_randao(state, &block.body).await?;
        
        // Process ETH1 data
        self.process_eth1_data(state, &block.body).await?;
        
        // Process operations
        self.process_operations(state, &block.body).await?;
        
        Ok(())
    }
}
```

### 2. Fork Choice Algorithm

LMD-GHOST (Latest Message Driven Greediest Heaviest Observed SubTree) implementation.

```rust
pub struct ForkChoice {
    justified_checkpoint: Checkpoint,
    finalized_checkpoint: Checkpoint,
    blocks: HashMap<Root, BlockInfo>,
    votes: HashMap<ValidatorIndex, Vote>,
}

impl ForkChoice {
    pub fn get_head(&self, justified_root: Root) -> Result<Root, ForkChoiceError> {
        let mut current_root = justified_root;
        
        loop {
            let children = self.get_children(current_root)?;
            if children.is_empty() {
                return Ok(current_root);
            }
            
            // Find child with highest weight
            let best_child = children
                .into_iter()
                .max_by_key(|&child| self.get_weight(child))
                .unwrap();
                
            current_root = best_child;
        }
    }
    
    fn get_weight(&self, block_root: Root) -> u64 {
        // Calculate weight based on attestations
        self.votes
            .values()
            .filter(|vote| self.is_descendant(vote.current_root, block_root))
            .map(|vote| vote.balance)
            .sum()
    }
}
```

### 3. Block Processing

#### Block Validation
```rust
pub struct BlockValidator {
    config: ChainConfig,
    signature_verifier: SignatureVerifier,
}

impl BlockValidator {
    pub async fn validate_block(
        &self,
        state: &BeaconState,
        block: &BeaconBlock,
    ) -> Result<(), ValidationError> {
        // Validate block slot
        self.validate_slot(state, block.slot)?;
        
        // Validate proposer
        self.validate_proposer(state, block)?;
        
        // Validate parent root
        self.validate_parent_root(state, &block.parent_root)?;
        
        // Validate state root
        self.validate_state_root(state, &block.state_root)?;
        
        // Validate body
        self.validate_body(state, &block.body).await?;
        
        Ok(())
    }
    
    async fn validate_body(
        &self,
        state: &BeaconState,
        body: &BeaconBlockBody,
    ) -> Result<(), ValidationError> {
        // Validate RANDAO reveal
        self.validate_randao_reveal(state, &body.randao_reveal).await?;
        
        // Validate proposer slashings
        for slashing in &body.proposer_slashings {
            self.validate_proposer_slashing(state, slashing).await?;
        }
        
        // Validate attester slashings
        for slashing in &body.attester_slashings {
            self.validate_attester_slashing(state, slashing).await?;
        }
        
        // Validate attestations
        for attestation in &body.attestations {
            self.validate_attestation(state, attestation).await?;
        }
        
        // Validate deposits
        for deposit in &body.deposits {
            self.validate_deposit(state, deposit)?;
        }
        
        // Validate voluntary exits
        for exit in &body.voluntary_exits {
            self.validate_voluntary_exit(state, exit).await?;
        }
        
        Ok(())
    }
}
```

### 4. Attestation Processing

```rust
pub struct AttestationProcessor {
    aggregation_pool: AttestationPool,
    signature_verifier: SignatureVerifier,
}

impl AttestationProcessor {
    pub async fn process_attestation(
        &self,
        state: &BeaconState,
        attestation: &Attestation,
    ) -> Result<(), AttestationError> {
        // Validate attestation structure
        self.validate_attestation_structure(attestation)?;
        
        // Validate attestation data
        self.validate_attestation_data(state, &attestation.data)?;
        
        // Validate aggregation bits
        self.validate_aggregation_bits(state, attestation)?;
        
        // Verify aggregate signature
        self.verify_aggregate_signature(state, attestation).await?;
        
        // Add to aggregation pool
        self.aggregation_pool.add_attestation(attestation.clone()).await?;
        
        Ok(())
    }
    
    pub async fn aggregate_attestations(
        &self,
        attestations: Vec<Attestation>,
    ) -> Result<Attestation, AttestationError> {
        // Group attestations by data
        let mut groups: HashMap<AttestationData, Vec<Attestation>> = HashMap::new();
        
        for attestation in attestations {
            groups.entry(attestation.data.clone())
                .or_default()
                .push(attestation);
        }
        
        // Aggregate each group
        let mut aggregated = Vec::new();
        for (data, group) in groups {
            let agg = self.aggregate_group(data, group)?;
            aggregated.push(agg);
        }
        
        // Return best aggregation
        aggregated.into_iter()
            .max_by_key(|att| att.aggregation_bits.count_ones())
            .ok_or(AttestationError::NoAttestations)
    }
}
```

### 5. Epoch Processing

```rust
pub struct EpochProcessor {
    config: ChainConfig,
}

impl EpochProcessor {
    pub fn process_epoch(
        &self,
        state: &mut BeaconState,
    ) -> Result<(), EpochProcessingError> {
        // Process justification and finalization
        self.process_justification_and_finalization(state)?;
        
        // Process rewards and penalties
        self.process_rewards_and_penalties(state)?;
        
        // Process registry updates
        self.process_registry_updates(state)?;
        
        // Process slashings
        self.process_slashings(state)?;
        
        // Process eth1 data reset
        self.process_eth1_data_reset(state)?;
        
        // Process effective balance updates
        self.process_effective_balance_updates(state)?;
        
        // Process slashings reset
        self.process_slashings_reset(state)?;
        
        // Process randao mixes reset
        self.process_randao_mixes_reset(state)?;
        
        // Process historical roots update
        self.process_historical_roots_update(state)?;
        
        // Process participation record updates
        self.process_participation_record_updates(state)?;
        
        Ok(())
    }
    
    fn process_justification_and_finalization(
        &self,
        state: &mut BeaconState,
    ) -> Result<(), EpochProcessingError> {
        let current_epoch = state.current_epoch();
        let previous_epoch = current_epoch.saturating_sub(1);
        
        // Calculate total active balance
        let total_active_balance = self.get_total_active_balance(state, current_epoch)?;
        
        // Calculate previous epoch target balance
        let previous_target_balance = self.get_matching_target_balance(
            state,
            previous_epoch,
        )?;
        
        // Calculate current epoch target balance
        let current_target_balance = self.get_matching_target_balance(
            state,
            current_epoch,
        )?;
        
        // Update justification bits
        state.justification_bits <<= 1;
        
        // Justify previous epoch
        if previous_target_balance * 3 >= total_active_balance * 2 {
            state.current_justified_checkpoint = Checkpoint {
                epoch: previous_epoch,
                root: self.get_block_root(state, previous_epoch)?,
            };
            state.justification_bits |= 0b10;
        }
        
        // Justify current epoch
        if current_target_balance * 3 >= total_active_balance * 2 {
            state.current_justified_checkpoint = Checkpoint {
                epoch: current_epoch,
                root: self.get_block_root(state, current_epoch)?,
            };
            state.justification_bits |= 0b01;
        }
        
        // Finalize checkpoints
        self.update_finalized_checkpoint(state)?;
        
        Ok(())
    }
}
```

## Validator Management

### Validator Lifecycle

```rust
pub struct ValidatorRegistry {
    validators: Vec<Validator>,
    balances: Vec<u64>,
    activation_queue: VecDeque<ValidatorIndex>,
    exit_queue: VecDeque<ValidatorIndex>,
}

impl ValidatorRegistry {
    pub fn add_validator(
        &mut self,
        pubkey: BLSPublicKey,
        withdrawal_credentials: Hash32,
        deposit_amount: u64,
    ) -> ValidatorIndex {
        let validator = Validator {
            pubkey,
            withdrawal_credentials,
            effective_balance: std::cmp::min(
                deposit_amount - (deposit_amount % EFFECTIVE_BALANCE_INCREMENT),
                MAX_EFFECTIVE_BALANCE,
            ),
            slashed: false,
            activation_eligibility_epoch: FAR_FUTURE_EPOCH,
            activation_epoch: FAR_FUTURE_EPOCH,
            exit_epoch: FAR_FUTURE_EPOCH,
            withdrawable_epoch: FAR_FUTURE_EPOCH,
        };
        
        let index = self.validators.len();
        self.validators.push(validator);
        self.balances.push(deposit_amount);
        
        index
    }
    
    pub fn process_validator_activation(
        &mut self,
        state: &BeaconState,
        validator_index: ValidatorIndex,
    ) -> Result<(), ValidatorError> {
        let validator = &mut self.validators[validator_index];
        let current_epoch = state.current_epoch();
        
        // Check activation eligibility
        if validator.activation_eligibility_epoch == FAR_FUTURE_EPOCH
            && validator.effective_balance == MAX_EFFECTIVE_BALANCE
        {
            validator.activation_eligibility_epoch = current_epoch + 1;
        }
        
        // Check activation
        if validator.activation_epoch == FAR_FUTURE_EPOCH
            && validator.activation_eligibility_epoch <= current_epoch
            && self.get_validator_churn_limit(state) > 0
        {
            validator.activation_epoch = self.compute_activation_exit_epoch(current_epoch);
        }
        
        Ok(())
    }
}
```

## Performance Optimizations

### Parallel Processing

```rust
pub struct ParallelConsensus {
    thread_pool: ThreadPool,
    signature_verifier: ParallelSignatureVerifier,
}

impl ParallelConsensus {
    pub async fn process_block_parallel(
        &self,
        state: &mut BeaconState,
        block: &BeaconBlock,
    ) -> Result<(), ConsensusError> {
        // Process operations in parallel
        let attestation_futures: Vec<_> = block.body.attestations
            .iter()
            .map(|att| self.verify_attestation_signature(state, att))
            .collect();
        
        let slashing_futures: Vec<_> = block.body.proposer_slashings
            .iter()
            .chain(block.body.attester_slashings.iter().map(|s| s as &dyn Slashing))
            .map(|slash| self.verify_slashing_signature(state, slash))
            .collect();
        
        // Wait for all signatures to verify
        let attestation_results = futures::future::join_all(attestation_futures).await;
        let slashing_results = futures::future::join_all(slashing_futures).await;
        
        // Check results
        for result in attestation_results.into_iter().chain(slashing_results) {
            result?;
        }
        
        // Apply state changes sequentially
        self.apply_state_changes(state, block).await?;
        
        Ok(())
    }
}
```

### Caching Strategies

```rust
pub struct ConsensusCache {
    state_cache: LruCache<Root, BeaconState>,
    committee_cache: LruCache<(Epoch, CommitteeIndex), Vec<ValidatorIndex>>,
    signature_cache: LruCache<SignatureKey, bool>,
}

impl ConsensusCache {
    pub fn get_committee(
        &mut self,
        state: &BeaconState,
        epoch: Epoch,
        committee_index: CommitteeIndex,
    ) -> Result<Vec<ValidatorIndex>, ConsensusError> {
        let key = (epoch, committee_index);
        
        if let Some(committee) = self.committee_cache.get(&key) {
            return Ok(committee.clone());
        }
        
        let committee = self.compute_committee(state, epoch, committee_index)?;
        self.committee_cache.put(key, committee.clone());
        
        Ok(committee)
    }
}
```

## Testing and Validation

### Consensus Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_state_transition() {
        let mut state = create_test_state();
        let block = create_test_block();
        
        let processor = StateProcessor::new(test_config());
        let result = processor.process_block(&mut state, &block).await;
        
        assert!(result.is_ok());
        assert_eq!(state.slot, block.slot);
    }
    
    #[tokio::test]
    async fn test_fork_choice() {
        let fork_choice = ForkChoice::new();
        
        // Add test blocks
        fork_choice.on_block(create_block(0, genesis_root())).await;
        fork_choice.on_block(create_block(1, block_root(0))).await;
        fork_choice.on_block(create_block(2, block_root(1))).await;
        
        // Add attestations
        fork_choice.on_attestation(create_attestation(1, block_root(1))).await;
        
        let head = fork_choice.get_head(genesis_root()).unwrap();
        assert_eq!(head, block_root(2));
    }
    
    #[test]
    fn test_validator_activation() {
        let mut registry = ValidatorRegistry::new();
        let state = create_test_state();
        
        let validator_index = registry.add_validator(
            test_pubkey(),
            test_withdrawal_credentials(),
            32_000_000_000, // 32 ETH
        );
        
        registry.process_validator_activation(&state, validator_index).unwrap();
        
        let validator = &registry.validators[validator_index];
        assert_ne!(validator.activation_eligibility_epoch, FAR_FUTURE_EPOCH);
    }
}
```

## Error Handling

### Consensus Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
    
    #[error("Fork choice error: {0}")]
    ForkChoice(#[from] ForkChoiceError),
    
    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),
    
    #[error("Validator error: {0}")]
    Validator(#[from] ValidatorError),
    
    #[error("Epoch processing error: {0}")]
    EpochProcessing(#[from] EpochProcessingError),
}
```

## Configuration

### Consensus Configuration

```toml
[consensus]
# Genesis configuration
genesis_time = 1606824000
genesis_fork_version = "0x00000000"
genesis_validators_root = "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"

# Timing parameters
seconds_per_slot = 12
slots_per_epoch = 32
epochs_per_eth1_voting_period = 64
slots_per_historical_root = 8192

# Validator parameters
max_committees_per_slot = 64
target_committee_size = 128
max_validators_per_committee = 2048
min_per_epoch_churn_limit = 4
churn_limit_quotient = 65536

# Rewards and penalties
base_reward_factor = 64
whistleblower_reward_quotient = 512
proposer_reward_quotient = 8
inactivity_penalty_quotient = 67108864

# Performance optimizations
enable_parallel_processing = true
signature_verification_threads = 8
committee_cache_size = 1000
state_cache_size = 100
```
