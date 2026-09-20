//! Metric family name constants by subsystem.

pub mod chain {
    pub const HEAD_SLOT: &str = "head_slot";
    pub const FINALIZED_SLOT: &str = "finalized_slot";
    pub const JUSTIFIED_SLOT: &str = "justified_slot";
    pub const SLOT_CURRENT: &str = "slot_current";
    pub const FINALITY_LAG: &str = "finality_lag_slots";
    pub const JUSTIFICATION_LAG: &str = "justification_lag_slots";
}

pub mod network {
    pub const PEER_COUNT: &str = "peer_count";
    pub const BOOTNODE_COUNT: &str = "bootnode_count";
}

pub mod storage {
    pub const SYNC_LAG: &str = "sync_lag_slots";
    pub const DURABLE_BLOCKS_FLUSHED: &str = "durable_blocks_flushed_total";
    pub const DURABLE_BLOCKS_PRUNED_FILES: &str = "durable_blocks_pruned_files_total";
    pub const DURABLE_BLOCKS_PRUNED_REDB: &str = "durable_blocks_pruned_redb_total";
    pub const DURABLE_BLOCKS_PRUNE_FLOOR: &str = "durable_blocks_prune_floor_slot";
    pub const DURABLE_BLOCKS_PRUNE_KEEP: &str = "durable_blocks_prune_keep_slots";
    pub const RANGE_SERVE_FOUND: &str = "blocks_by_range_serve_found_total";
    pub const RANGE_SERVE_MISSING: &str = "blocks_by_range_serve_missing_total";
    pub const RANGE_SERVE_CACHE_SLOTS: &str = "blocks_by_range_serve_cache_slots";
    pub const SERVE_CACHE_SEED_INDEXED: &str = "serve_cache_seed_indexed";
    pub const SERVE_CACHE_SEED_CANDIDATES: &str = "serve_cache_seed_candidates";
}

pub mod validator {
    pub const DUTY_SUPPRESSED: &str = "duty_suppressed_total";
    pub const VALIDATOR_COUNT: &str = "validator_count";
    pub const AGGREGATOR_ENABLED: &str = "aggregator_enabled";
    pub const LOCAL_FINALITY_ENABLED: &str = "local_finality_enabled";
}

pub mod prover {
    pub const TIMEOUT: &str = "prover_timeout_total";
}

pub mod readiness {
    pub const READY: &str = "ready";
    pub const STORAGE: &str = "ready_storage";
    pub const CRYPTO: &str = "ready_crypto";
    pub const SIGNER: &str = "ready_signer";
    pub const NETWORK: &str = "ready_network";
    pub const PROVER: &str = "ready_prover";
}
