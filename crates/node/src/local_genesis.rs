//! Recent multi-validator genesis for local long-run / finality smoke.

use ethean_primitives::Bytes52;
use ethean_genesis::{BuiltGenesis, GenesisBuilder, GenesisError};
use std::time::{SystemTime, UNIX_EPOCH};

/// Genesis time a few slots in the past so wall-clock duties start near slot 0–N
/// (avoids hashing millions of empty slots from a 2023-era smoke timestamp).
pub fn recent_genesis_time_secs(lookback_slots: u64, seconds_per_slot: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1_700_000_000);
    let back = lookback_slots.saturating_mul(seconds_per_slot.max(1));
    now.saturating_sub(back.max(8))
}

/// Build a local genesis with `validator_count` zero-key validators and a recent time.
pub fn local_devnet_genesis(
    validator_count: usize,
    seconds_per_slot: u64,
) -> Result<BuiltGenesis, GenesisError> {
    let n = validator_count.max(1);
    let genesis_time = recent_genesis_time_secs(4, seconds_per_slot);
    let mut builder = GenesisBuilder::new(genesis_time);
    for _ in 0..n {
        builder = builder.push_validator(Bytes52::ZERO, Bytes52::ZERO);
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_n_validators_with_fresh_time() {
        let g = local_devnet_genesis(4, 4).unwrap();
        assert_eq!(g.state.validators.len(), 4);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(g.state.config.genesis_time + 120 >= now);
    }
}
