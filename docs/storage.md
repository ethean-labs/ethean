# Storage Layer Documentation

## Overview

Panro's storage layer provides efficient, reliable data persistence for blockchain state, blocks, attestations, and metadata. The system is designed for high performance with support for concurrent reads/writes, automatic compaction, and robust backup strategies.

## Storage Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Storage Layer                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ State       │  │ Block       │  │ Attestation         │ │
│  │ Database    │  │ Storage     │  │ Storage             │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Cache       │  │ Index       │  │ Backup &            │ │
│  │ Layer       │  │ Manager     │  │ Recovery            │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Database Engine                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ RocksDB     │  │ Transaction │  │ Compaction          │ │
│  │ Engine      │  │ Manager     │  │ Strategy            │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Database Schema

### 1. State Storage

The beacon state is stored using a merkle tree structure for efficient state root computation and historical state access.

```rust
use rocksdb::{DB, Options, ColumnFamily, WriteBatch};
use serde::{Serialize, Deserialize};

pub struct StateStorage {
    db: Arc<DB>,
    state_cache: LruCache<StateRoot, BeaconState>,
    finalized_states: LruCache<Epoch, StateRoot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredState {
    pub slot: Slot,
    pub state_root: StateRoot,
    pub state_data: Vec<u8>,  // SSZ encoded state
    pub is_finalized: bool,
    pub created_at: u64,
}

impl StateStorage {
    const STATES_CF: &'static str = "states";
    const STATE_ROOTS_CF: &'static str = "state_roots";
    const FINALIZED_STATES_CF: &'static str = "finalized_states";
    const STATE_METADATA_CF: &'static str = "state_metadata";
    
    pub fn new(db_path: &Path) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        // Configure for optimal state storage
        opts.set_max_open_files(1000);
        opts.set_use_fsync(false);  // Use fdatasync for better performance
        opts.set_bytes_per_sync(1048576);  // 1MB
        opts.set_wal_bytes_per_sync(1048576);
        
        // Block-based table options
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_size(32 * 1024);  // 32KB blocks
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        opts.set_block_based_table_factory(&block_opts);
        
        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new(Self::STATES_CF, opts.clone()),
            ColumnFamilyDescriptor::new(Self::STATE_ROOTS_CF, opts.clone()),
            ColumnFamilyDescriptor::new(Self::FINALIZED_STATES_CF, opts.clone()),
            ColumnFamilyDescriptor::new(Self::STATE_METADATA_CF, opts.clone()),
        ];
        
        let db = DB::open_cf_descriptors(&opts, db_path, cf_descriptors)?;
        
        Ok(Self {
            db: Arc::new(db),
            state_cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
            finalized_states: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        })
    }
    
    pub async fn store_state(
        &mut self,
        state: &BeaconState,
        state_root: StateRoot,
    ) -> Result<(), StorageError> {
        let slot = state.slot();
        let is_finalized = self.is_state_finalized(slot).await?;
        
        // Serialize state
        let state_data = state.as_ssz_bytes();
        
        let stored_state = StoredState {
            slot,
            state_root,
            state_data,
            is_finalized,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Prepare batch write
        let mut batch = WriteBatch::default();
        
        // Store state data
        let state_key = Self::state_key(state_root);
        let state_value = bincode::serialize(&stored_state)?;
        batch.put_cf(
            self.db.cf_handle(Self::STATES_CF).unwrap(),
            state_key,
            state_value,
        );
        
        // Store state root mapping
        let slot_key = Self::slot_key(slot);
        batch.put_cf(
            self.db.cf_handle(Self::STATE_ROOTS_CF).unwrap(),
            slot_key,
            state_root.as_bytes(),
        );
        
        // Store finalized state if applicable
        if is_finalized {
            let epoch = slot.epoch(SLOTS_PER_EPOCH);
            let epoch_key = Self::epoch_key(epoch);
            batch.put_cf(
                self.db.cf_handle(Self::FINALIZED_STATES_CF).unwrap(),
                epoch_key,
                state_root.as_bytes(),
            );
            
            // Cache finalized state
            self.finalized_states.put(epoch, state_root);
        }
        
        // Execute batch
        self.db.write(batch)?;
        
        // Update cache
        self.state_cache.put(state_root, state.clone());
        
        Ok(())
    }
    
    pub async fn get_state(
        &mut self,
        state_root: StateRoot,
    ) -> Result<Option<BeaconState>, StorageError> {
        // Check cache first
        if let Some(state) = self.state_cache.get(&state_root) {
            return Ok(Some(state.clone()));
        }
        
        // Load from database
        let state_key = Self::state_key(state_root);
        let cf_handle = self.db.cf_handle(Self::STATES_CF).unwrap();
        
        if let Some(value) = self.db.get_cf(cf_handle, state_key)? {
            let stored_state: StoredState = bincode::deserialize(&value)?;
            let state = BeaconState::from_ssz_bytes(&stored_state.state_data)?;
            
            // Update cache
            self.state_cache.put(state_root, state.clone());
            
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
    
    fn state_key(state_root: StateRoot) -> [u8; 32] {
        state_root.0
    }
    
    fn slot_key(slot: Slot) -> [u8; 8] {
        slot.as_u64().to_be_bytes()
    }
    
    fn epoch_key(epoch: Epoch) -> [u8; 8] {
        epoch.as_u64().to_be_bytes()
    }
}
```

### 2. Block Storage

Efficient storage and retrieval of beacon blocks with support for range queries and fork tracking.

```rust
pub struct BlockStorage {
    db: Arc<DB>,
    block_cache: LruCache<BlockRoot, SignedBeaconBlock>,
    fork_tracker: ForkTracker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBlock {
    pub slot: Slot,
    pub block_root: BlockRoot,
    pub parent_root: BlockRoot,
    pub state_root: StateRoot,
    pub block_data: Vec<u8>,  // SSZ encoded block
    pub canonical: bool,
    pub finalized: bool,
    pub imported_at: u64,
}

impl BlockStorage {
    const BLOCKS_CF: &'static str = "blocks";
    const BLOCK_ROOTS_CF: &'static str = "block_roots";
    const SLOT_TO_BLOCK_CF: &'static str = "slot_to_block";
    const CANONICAL_CHAIN_CF: &'static str = "canonical_chain";
    const FORK_METADATA_CF: &'static str = "fork_metadata";
    
    pub async fn store_block(
        &mut self,
        block: &SignedBeaconBlock,
        block_root: BlockRoot,
        is_canonical: bool,
    ) -> Result<(), StorageError> {
        let slot = block.message().slot();
        let parent_root = block.message().parent_root();
        let state_root = block.message().state_root();
        
        // Check if block is finalized
        let is_finalized = self.is_block_finalized(slot).await?;
        
        let stored_block = StoredBlock {
            slot,
            block_root,
            parent_root,
            state_root,
            block_data: block.as_ssz_bytes(),
            canonical: is_canonical,
            finalized: is_finalized,
            imported_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let mut batch = WriteBatch::default();
        
        // Store block data
        let block_key = Self::block_key(block_root);
        let block_value = bincode::serialize(&stored_block)?;
        batch.put_cf(
            self.db.cf_handle(Self::BLOCKS_CF).unwrap(),
            block_key,
            block_value,
        );
        
        // Store slot to block mapping
        let slot_key = Self::slot_key(slot);
        batch.put_cf(
            self.db.cf_handle(Self::SLOT_TO_BLOCK_CF).unwrap(),
            slot_key,
            block_root.as_bytes(),
        );
        
        // Update canonical chain if needed
        if is_canonical {
            batch.put_cf(
                self.db.cf_handle(Self::CANONICAL_CHAIN_CF).unwrap(),
                slot_key,
                block_root.as_bytes(),
            );
        }
        
        // Execute batch
        self.db.write(batch)?;
        
        // Update cache and fork tracker
        self.block_cache.put(block_root, block.clone());
        self.fork_tracker.add_block(block_root, parent_root, slot, is_canonical);
        
        Ok(())
    }
    
    pub async fn get_blocks_by_range(
        &self,
        start_slot: Slot,
        count: u64,
        step: u64,
    ) -> Result<Vec<SignedBeaconBlock>, StorageError> {
        let mut blocks = Vec::new();
        let mut current_slot = start_slot;
        let mut fetched = 0;
        
        while fetched < count {
            if let Some(block) = self.get_block_by_slot(current_slot).await? {
                blocks.push(block);
                fetched += 1;
            }
            
            current_slot += step;
            
            // Prevent infinite loops
            if current_slot.as_u64() > start_slot.as_u64() + (count * step * 2) {
                break;
            }
        }
        
        Ok(blocks)
    }
    
    pub async fn get_block_by_slot(
        &self,
        slot: Slot,
    ) -> Result<Option<SignedBeaconBlock>, StorageError> {
        let slot_key = Self::slot_key(slot);
        let cf_handle = self.db.cf_handle(Self::SLOT_TO_BLOCK_CF).unwrap();
        
        if let Some(block_root_bytes) = self.db.get_cf(cf_handle, slot_key)? {
            let block_root = BlockRoot::from_slice(&block_root_bytes);
            self.get_block(block_root).await
        } else {
            Ok(None)
        }
    }
    
    pub async fn prune_old_blocks(&mut self, retain_epochs: u64) -> Result<u64, StorageError> {
        let current_epoch = self.get_current_epoch().await?;
        let cutoff_epoch = current_epoch.saturating_sub(retain_epochs);
        let cutoff_slot = cutoff_epoch * SLOTS_PER_EPOCH;
        
        let mut deleted_count = 0;
        let mut batch = WriteBatch::default();
        
        // Iterate through blocks and mark old ones for deletion
        let cf_handle = self.db.cf_handle(Self::BLOCKS_CF).unwrap();
        let iter = self.db.iterator_cf(cf_handle, IteratorMode::Start);
        
        for item in iter {
            let (key, value) = item?;
            let stored_block: StoredBlock = bincode::deserialize(&value)?;
            
            // Only delete non-finalized old blocks
            if stored_block.slot < cutoff_slot && !stored_block.finalized {
                batch.delete_cf(cf_handle, &key);
                deleted_count += 1;
                
                // Also remove from auxiliary indexes
                let slot_key = Self::slot_key(stored_block.slot);
                batch.delete_cf(
                    self.db.cf_handle(Self::SLOT_TO_BLOCK_CF).unwrap(),
                    slot_key,
                );
                
                // Remove from cache
                self.block_cache.pop(&stored_block.block_root);
            }
        }
        
        // Execute batch deletion
        self.db.write(batch)?;
        
        tracing::info!("Pruned {} old blocks", deleted_count);
        Ok(deleted_count)
    }
}
```

### 3. Attestation Storage

High-performance storage for attestations with efficient aggregation and committee-based indexing.

```rust
pub struct AttestationStorage {
    db: Arc<DB>,
    aggregated_attestations: LruCache<AttestationKey, AggregateAttestation>,
    committee_cache: LruCache<(Slot, CommitteeIndex), Vec<ValidatorIndex>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AttestationKey {
    pub slot: Slot,
    pub committee_index: CommitteeIndex,
    pub target_epoch: Epoch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAttestation {
    pub attestation: Attestation,
    pub aggregation_bits: Bitfield,
    pub committee_index: CommitteeIndex,
    pub slot: Slot,
    pub target_epoch: Epoch,
    pub target_root: Root,
    pub source_epoch: Epoch,
    pub source_root: Root,
    pub stored_at: u64,
}

impl AttestationStorage {
    const ATTESTATIONS_CF: &'static str = "attestations";
    const SLOT_ATTESTATIONS_CF: &'static str = "slot_attestations";
    const COMMITTEE_ATTESTATIONS_CF: &'static str = "committee_attestations";
    const AGGREGATED_ATTESTATIONS_CF: &'static str = "aggregated_attestations";
    
    pub async fn store_attestation(
        &mut self,
        attestation: &Attestation,
    ) -> Result<(), StorageError> {
        let data = &attestation.data;
        let key = AttestationKey {
            slot: data.slot,
            committee_index: data.index,
            target_epoch: data.target.epoch,
        };
        
        let stored_attestation = StoredAttestation {
            attestation: attestation.clone(),
            aggregation_bits: attestation.aggregation_bits.clone(),
            committee_index: data.index,
            slot: data.slot,
            target_epoch: data.target.epoch,
            target_root: data.target.root,
            source_epoch: data.source.epoch,
            source_root: data.source.root,
            stored_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let mut batch = WriteBatch::default();
        
        // Store attestation
        let att_key = Self::attestation_key(&key);
        let att_value = bincode::serialize(&stored_attestation)?;
        batch.put_cf(
            self.db.cf_handle(Self::ATTESTATIONS_CF).unwrap(),
            att_key,
            att_value,
        );
        
        // Index by slot
        let slot_key = Self::slot_attestation_key(data.slot, &key);
        batch.put_cf(
            self.db.cf_handle(Self::SLOT_ATTESTATIONS_CF).unwrap(),
            slot_key,
            b"",
        );
        
        // Index by committee
        let committee_key = Self::committee_attestation_key(
            data.slot,
            data.index,
            &key,
        );
        batch.put_cf(
            self.db.cf_handle(Self::COMMITTEE_ATTESTATIONS_CF).unwrap(),
            committee_key,
            b"",
        );
        
        self.db.write(batch)?;
        
        // Try to aggregate with existing attestations
        self.try_aggregate_attestation(attestation).await?;
        
        Ok(())
    }
    
    async fn try_aggregate_attestation(
        &mut self,
        new_attestation: &Attestation,
    ) -> Result<(), StorageError> {
        let data = &new_attestation.data;
        let key = AttestationKey {
            slot: data.slot,
            committee_index: data.index,
            target_epoch: data.target.epoch,
        };
        
        // Check for existing aggregated attestation
        if let Some(mut existing) = self.aggregated_attestations.get(&key).cloned() {
            // Try to aggregate
            if let Ok(aggregated) = existing.aggregate(new_attestation) {
                self.aggregated_attestations.put(key.clone(), aggregated.clone());
                
                // Store updated aggregate
                let agg_key = Self::aggregated_attestation_key(&key);
                let agg_value = bincode::serialize(&aggregated)?;
                self.db.put_cf(
                    self.db.cf_handle(Self::AGGREGATED_ATTESTATIONS_CF).unwrap(),
                    agg_key,
                    agg_value,
                )?;
            }
        } else {
            // Create new aggregate
            let aggregate = AggregateAttestation::from_attestation(new_attestation);
            self.aggregated_attestations.put(key.clone(), aggregate.clone());
            
            let agg_key = Self::aggregated_attestation_key(&key);
            let agg_value = bincode::serialize(&aggregate)?;
            self.db.put_cf(
                self.db.cf_handle(Self::AGGREGATED_ATTESTATIONS_CF).unwrap(),
                agg_key,
                agg_value,
            )?;
        }
        
        Ok(())
    }
    
    pub async fn get_attestations_by_slot(
        &self,
        slot: Slot,
    ) -> Result<Vec<Attestation>, StorageError> {
        let mut attestations = Vec::new();
        let prefix = Self::slot_prefix(slot);
        
        let cf_handle = self.db.cf_handle(Self::SLOT_ATTESTATIONS_CF).unwrap();
        let iter = self.db.prefix_iterator_cf(cf_handle, prefix);
        
        for item in iter {
            let (key, _) = item?;
            if let Some(att_key) = Self::decode_slot_attestation_key(&key) {
                if let Some(attestation) = self.get_attestation(&att_key).await? {
                    attestations.push(attestation.attestation);
                }
            }
        }
        
        Ok(attestations)
    }
    
    fn attestation_key(key: &AttestationKey) -> Vec<u8> {
        let mut result = Vec::with_capacity(24);
        result.extend_from_slice(&key.slot.as_u64().to_be_bytes());
        result.extend_from_slice(&key.committee_index.as_u64().to_be_bytes());
        result.extend_from_slice(&key.target_epoch.as_u64().to_be_bytes());
        result
    }
}
```

## Cache Management

### Multi-Level Caching

```rust
pub struct CacheManager {
    l1_cache: Arc<RwLock<LruCache<CacheKey, CacheValue>>>,
    l2_cache: Arc<RwLock<LruCache<CacheKey, CacheValue>>>,
    cache_stats: CacheStats,
    background_loader: BackgroundLoader,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CacheKey {
    State(StateRoot),
    Block(BlockRoot),
    Attestation(AttestationKey),
    ValidatorSet(Epoch),
    Committee(Slot, CommitteeIndex),
}

#[derive(Debug, Clone)]
pub enum CacheValue {
    State(Arc<BeaconState>),
    Block(Arc<SignedBeaconBlock>),
    Attestation(Arc<Attestation>),
    ValidatorSet(Arc<Vec<Validator>>),
    Committee(Arc<Vec<ValidatorIndex>>),
}

impl CacheManager {
    pub fn new(l1_size: usize, l2_size: usize) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(l1_size).unwrap())
            )),
            l2_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(l2_size).unwrap())
            )),
            cache_stats: CacheStats::default(),
            background_loader: BackgroundLoader::new(),
        }
    }
    
    pub async fn get<T>(&self, key: &CacheKey) -> Option<T>
    where
        T: TryFrom<CacheValue> + Clone + 'static,
    {
        // Try L1 cache first
        {
            let mut l1 = self.l1_cache.write().await;
            if let Some(value) = l1.get(key) {
                self.cache_stats.record_hit(CacheLevel::L1);
                if let Ok(typed_value) = T::try_from(value.clone()) {
                    return Some(typed_value);
                }
            }
        }
        
        // Try L2 cache
        {
            let mut l2 = self.l2_cache.write().await;
            if let Some(value) = l2.get(key) {
                self.cache_stats.record_hit(CacheLevel::L2);
                
                // Promote to L1
                let mut l1 = self.l1_cache.write().await;
                l1.put(key.clone(), value.clone());
                
                if let Ok(typed_value) = T::try_from(value.clone()) {
                    return Some(typed_value);
                }
            }
        }
        
        self.cache_stats.record_miss();
        None
    }
    
    pub async fn put(&self, key: CacheKey, value: CacheValue) {
        let mut l1 = self.l1_cache.write().await;
        
        // If L1 is full, evict to L2
        if l1.len() >= l1.cap().get() {
            if let Some((evicted_key, evicted_value)) = l1.pop_lru() {
                let mut l2 = self.l2_cache.write().await;
                l2.put(evicted_key, evicted_value);
            }
        }
        
        l1.put(key, value);
    }
    
    pub async fn preload_committee_cache(&self, epoch: Epoch) {
        let key = CacheKey::ValidatorSet(epoch);
        
        // Check if already cached
        if self.get::<Arc<Vec<Validator>>>(&key).await.is_some() {
            return;
        }
        
        // Load in background
        self.background_loader.load_validator_set(epoch).await;
    }
    
    pub fn get_stats(&self) -> CacheStats {
        self.cache_stats.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub l1_hits: AtomicU64,
    pub l2_hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
}

impl CacheStats {
    pub fn hit_ratio(&self) -> f64 {
        let total_hits = self.l1_hits.load(Ordering::Relaxed) 
            + self.l2_hits.load(Ordering::Relaxed);
        let total_requests = total_hits + self.misses.load(Ordering::Relaxed);
        
        if total_requests == 0 {
            0.0
        } else {
            total_hits as f64 / total_requests as f64
        }
    }
}
```

## Backup and Recovery

### Automated Backup System

```rust
pub struct BackupManager {
    storage_path: PathBuf,
    backup_path: PathBuf,
    backup_config: BackupConfig,
    scheduler: BackupScheduler,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub enabled: bool,
    pub interval: Duration,
    pub retention_days: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub max_backup_size: u64,
    pub incremental_enabled: bool,
}

impl BackupManager {
    pub async fn start_automated_backups(&mut self) -> Result<(), BackupError> {
        if !self.backup_config.enabled {
            return Ok(());
        }
        
        let interval = self.backup_config.interval;
        let backup_path = self.backup_path.clone();
        let storage_path = self.storage_path.clone();
        let config = self.backup_config.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                match Self::create_backup(&storage_path, &backup_path, &config).await {
                    Ok(backup_info) => {
                        tracing::info!(
                            "Backup created successfully: {} (size: {} MB)",
                            backup_info.filename,
                            backup_info.size / 1024 / 1024
                        );
                    }
                    Err(error) => {
                        tracing::error!("Backup failed: {}", error);
                    }
                }
                
                // Clean up old backups
                if let Err(error) = Self::cleanup_old_backups(&backup_path, &config).await {
                    tracing::error!("Backup cleanup failed: {}", error);
                }
            }
        });
        
        Ok(())
    }
    
    async fn create_backup(
        storage_path: &Path,
        backup_path: &Path,
        config: &BackupConfig,
    ) -> Result<BackupInfo, BackupError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_filename = if config.incremental_enabled {
            format!("panro_incremental_backup_{}.tar.gz", timestamp)
        } else {
            format!("panro_full_backup_{}.tar.gz", timestamp)
        };
        
        let backup_file_path = backup_path.join(&backup_filename);
        
        // Create backup directory if it doesn't exist
        tokio::fs::create_dir_all(backup_path).await?;
        
        let start_time = Instant::now();
        
        // Create tar archive
        let tar_gz = File::create(&backup_file_path).await?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);
        
        // Add database files
        tar.append_dir_all("db", storage_path)?;
        
        // Add configuration files
        if let Some(config_path) = Self::find_config_file(storage_path) {
            tar.append_path_with_name(&config_path, "config.toml")?;
        }
        
        // Finalize archive
        tar.finish()?;
        
        let backup_size = tokio::fs::metadata(&backup_file_path).await?.len();
        let duration = start_time.elapsed();
        
        // Encrypt if enabled
        if config.encryption_enabled {
            Self::encrypt_backup(&backup_file_path).await?;
        }
        
        let backup_info = BackupInfo {
            filename: backup_filename,
            size: backup_size,
            created_at: timestamp,
            duration,
            backup_type: if config.incremental_enabled {
                BackupType::Incremental
            } else {
                BackupType::Full
            },
            encrypted: config.encryption_enabled,
        };
        
        // Save backup metadata
        Self::save_backup_metadata(&backup_path, &backup_info).await?;
        
        Ok(backup_info)
    }
    
    pub async fn restore_from_backup(
        &self,
        backup_file: &Path,
        target_path: &Path,
    ) -> Result<(), BackupError> {
        tracing::info!("Starting restore from backup: {:?}", backup_file);
        
        // Decrypt if needed
        let restore_file = if self.backup_config.encryption_enabled {
            let decrypted_path = backup_file.with_extension("decrypted");
            Self::decrypt_backup(backup_file, &decrypted_path).await?;
            decrypted_path
        } else {
            backup_file.to_path_buf()
        };
        
        // Extract backup
        let tar_gz = File::open(&restore_file).await?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        
        // Create target directory
        tokio::fs::create_dir_all(target_path).await?;
        
        // Extract files
        archive.unpack(target_path)?;
        
        // Verify restoration
        Self::verify_restored_data(target_path).await?;
        
        // Clean up temporary files
        if self.backup_config.encryption_enabled {
            tokio::fs::remove_file(&restore_file).await?;
        }
        
        tracing::info!("Backup restoration completed successfully");
        Ok(())
    }
    
    async fn verify_restored_data(path: &Path) -> Result<(), BackupError> {
        // Check if database can be opened
        let db_path = path.join("db");
        if !db_path.exists() {
            return Err(BackupError::InvalidBackup(
                "Database directory not found".to_string()
            ));
        }
        
        // Try to open database
        let opts = Options::default();
        let _db = DB::open_for_read_only(&opts, &db_path, false)
            .map_err(|e| BackupError::DatabaseError(e.to_string()))?;
        
        // Additional integrity checks could be added here
        
        Ok(())
    }
}
```

## Performance Optimization

### Database Tuning

```rust
pub struct DatabaseOptimizer {
    db: Arc<DB>,
    metrics: DatabaseMetrics,
    compaction_scheduler: CompactionScheduler,
}

impl DatabaseOptimizer {
    pub async fn optimize_for_workload(&mut self, workload: WorkloadType) -> Result<(), OptimizationError> {
        match workload {
            WorkloadType::ReadHeavy => {
                self.optimize_for_reads().await?;
            }
            WorkloadType::WriteHeavy => {
                self.optimize_for_writes().await?;
            }
            WorkloadType::Balanced => {
                self.optimize_for_balanced().await?;
            }
        }
        
        Ok(())
    }
    
    async fn optimize_for_reads(&mut self) -> Result<(), OptimizationError> {
        // Increase block cache size
        let mut opts = Options::default();
        opts.set_block_cache_size(512 * 1024 * 1024);  // 512MB
        
        // Use more aggressive caching
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_bloom_filter(10.0, false);
        
        // Enable prefetching
        opts.set_advise_random_on_open(false);
        
        Ok(())
    }
    
    async fn optimize_for_writes(&mut self) -> Result<(), OptimizationError> {
        // Increase write buffer size
        let mut opts = Options::default();
        opts.set_write_buffer_size(128 * 1024 * 1024);  // 128MB
        opts.set_max_write_buffer_number(6);
        opts.set_min_write_buffer_number_to_merge(2);
        
        // Optimize compaction
        opts.set_level_zero_file_num_compaction_trigger(8);
        opts.set_level_zero_slowdown_writes_trigger(20);
        opts.set_level_zero_stop_writes_trigger(36);
        
        // Use faster compression for lower levels
        opts.set_compression_per_level(&[
            DBCompressionType::None,    // L0
            DBCompressionType::None,    // L1
            DBCompressionType::Snappy,  // L2+
        ]);
        
        Ok(())
    }
    
    pub async fn monitor_performance(&mut self) -> DatabaseMetrics {
        let stats = self.db.property_value("rocksdb.stats").unwrap_or_default();
        let cache_usage = self.db.property_int_value("rocksdb.block-cache-usage")
            .unwrap_or(0);
        let memtable_usage = self.db.property_int_value("rocksdb.cur-size-all-mem-tables")
            .unwrap_or(0);
        
        DatabaseMetrics {
            cache_hit_ratio: self.calculate_cache_hit_ratio(),
            write_amplification: self.calculate_write_amplification(),
            read_amplification: self.calculate_read_amplification(),
            cache_usage_bytes: cache_usage,
            memtable_usage_bytes: memtable_usage,
            compaction_pending: self.get_pending_compaction_count(),
            stats_text: stats,
        }
    }
}
```

## Configuration

### Storage Configuration

```toml
[storage]
# Database settings
db_path = "./data/db"
cache_size_mb = 512
write_buffer_size_mb = 128
max_open_files = 1000

# Compaction settings
level0_file_num_compaction_trigger = 8
level0_slowdown_writes_trigger = 20
level0_stop_writes_trigger = 36
max_background_compactions = 4
max_background_flushes = 2

# Cache settings
[storage.cache]
l1_size = 1000
l2_size = 10000
preload_committees = true
preload_validator_sets = true

# Backup settings
[storage.backup]
enabled = true
interval = "6h"
retention_days = 30
compression_enabled = true
encryption_enabled = false
max_backup_size_gb = 10
incremental_enabled = true
backup_path = "./backups"

# Pruning settings
[storage.pruning]
enabled = true
retain_epochs = 256
prune_interval = "24h"
keep_finalized_states = true
```
