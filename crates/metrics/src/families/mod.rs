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
}

pub mod storage {
    pub const SYNC_LAG: &str = "sync_lag_slots";
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
