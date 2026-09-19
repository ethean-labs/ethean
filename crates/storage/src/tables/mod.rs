//! Typed table names and key helpers (SSZ values, not JSON).

use ethean_primitives::Hash32;

/// Immutable block SSZ by root.
pub const TABLE_BLOCKS: &str = "blocks";

/// Post-state SSZ by block root.
pub const TABLE_STATES: &str = "states";

/// Parent root index: child → parent.
pub const TABLE_PARENT: &str = "indices.parent";

/// Slot → root index (canonical preference recorded separately).
pub const TABLE_SLOT_ROOT: &str = "indices.slot_root";

/// Canonical metadata (head, justified, finalized).
pub const TABLE_METADATA: &str = "metadata";

/// Latest votes / attestation payloads.
pub const TABLE_VOTES: &str = "votes";

/// Bounded proof / aggregate pools.
pub const TABLE_POOLS: &str = "pools";

/// Encode a Hash32 as a key.
pub fn root_key(root: &Hash32) -> Vec<u8> {
    root.to_vec()
}

/// Encode a slot as little-endian key.
pub fn slot_key(slot: u64) -> Vec<u8> {
    slot.to_le_bytes().to_vec()
}

/// Metadata key for canonical head root.
pub const META_HEAD: &[u8] = b"head";

/// Metadata key for finalized checkpoint root.
pub const META_FINALIZED: &[u8] = b"finalized";

/// Metadata key for justified checkpoint root.
pub const META_JUSTIFIED: &[u8] = b"justified";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_key_roundtrip_prefix() {
        assert_eq!(slot_key(1).len(), 8);
        assert_eq!(root_key(&[9u8; 32]).len(), 32);
    }
}
