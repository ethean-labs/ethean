//! Metric family name constants by subsystem.

pub mod chain {
    pub const HEAD_SLOT: &str = "head_slot";
    pub const FINALIZED_SLOT: &str = "finalized_slot";
}

pub mod network {
    pub const PEER_COUNT: &str = "peer_count";
}

pub mod storage {
    pub const SYNC_LAG: &str = "sync_lag_slots";
}

pub mod validator {
    pub const DUTY_SUPPRESSED: &str = "duty_suppressed_total";
}

pub mod prover {
    pub const TIMEOUT: &str = "prover_timeout_total";
}
