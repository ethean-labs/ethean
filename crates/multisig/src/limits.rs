//! Consensus and circuit limits enforced before any proof reaches leanMultisig.

/// leanSpec `ByteList512KiB` bound on every proof carried on the wire.
pub const MAX_PROOF_BYTES: usize = 512 * 1024;

/// Upper bound on the LZ4 expansion factor of a well-formed proof. Proofs are
/// high-entropy and compress to roughly 1x; leanVM uses the same factor.
pub const MAX_DECOMPRESS_RATIO: usize = 8;

/// Components one Type-2 proof can merge (leanVM `MAX_RECURSIONS`).
pub const MAX_COMPONENTS: usize = lean_multisig::MAX_RECURSIONS;

/// Child proofs one Type-1 aggregation can absorb (leanVM `MAX_RECURSIONS`).
pub const MAX_CHILDREN: usize = lean_multisig::MAX_RECURSIONS;

/// Public keys one component can bind (leanVM `MAX_XMSS_AGGREGATED`).
pub const MAX_KEYS_PER_COMPONENT: usize = lean_multisig::MAX_XMSS_AGGREGATED;

/// WHIR `log2(1 / rate)` used on pq-devnet-4 by every client.
pub const LOG_INV_RATE: usize = 2;

/// Reject empty or oversized proof bytes.
pub fn check_proof_len(proof: &[u8]) -> crate::Result<()> {
    if proof.is_empty() || proof.len() > MAX_PROOF_BYTES {
        return Err(crate::MultisigError::ProofLength {
            len: proof.len(),
            max: MAX_PROOF_BYTES,
        });
    }
    Ok(())
}

/// Reject proofs whose LZ4 size prefix would force an oversized allocation.
///
/// leanMultisig's `decompress_without_pubkeys` trusts this prefix, so a
/// 512 KiB proof could otherwise request gigabytes.
pub fn check_decompressed_size(proof: &[u8]) -> crate::Result<()> {
    let Some(prefix) = proof.get(..4) else {
        return Err(crate::MultisigError::Malformed);
    };
    let declared = u32::from_le_bytes(prefix.try_into().expect("4-byte prefix")) as usize;
    let max = proof.len().saturating_mul(MAX_DECOMPRESS_RATIO);
    if declared > max {
        return Err(crate::MultisigError::DecompressionBound { declared, max });
    }
    Ok(())
}

/// Convert a consensus slot to the 32-bit XMSS epoch.
pub fn slot_to_epoch(slot: u64) -> crate::Result<u32> {
    u32::try_from(slot).map_err(|_| crate::MultisigError::SlotOutOfRange(slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inflated_size_prefix() {
        let mut proof = vec![0u8; 64];
        proof[..4].copy_from_slice(&(64u32 * 8).to_le_bytes());
        assert!(check_decompressed_size(&proof).is_ok());
        proof[..4].copy_from_slice(&(64u32 * 8 + 1).to_le_bytes());
        assert!(check_decompressed_size(&proof).is_err());
        assert!(check_decompressed_size(&[1, 2, 3]).is_err());
    }

    #[test]
    fn proof_length_and_slot_bounds() {
        assert!(check_proof_len(&[]).is_err());
        assert!(check_proof_len(&vec![0u8; MAX_PROOF_BYTES]).is_ok());
        assert!(check_proof_len(&vec![0u8; MAX_PROOF_BYTES + 1]).is_err());
        assert_eq!(slot_to_epoch(7).unwrap(), 7);
        assert!(slot_to_epoch(1 << 32).is_err());
    }
}
