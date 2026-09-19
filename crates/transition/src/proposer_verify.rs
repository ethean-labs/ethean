//! Optional proposer XMSS verify against the registry proposal public key.

use ethean_crypto::{verify, CryptoBackend, PublicKey, Signature, SIGNATURE_BYTES};
use ethean_types::{Block, State};

use crate::error::TransitionError;

/// Verify a proposer signature over the block tree root using `state` registry keys.
///
/// Sidecar path: signature is not part of `SignedBlock.proof`. Callers supply it
/// when gossip carries a binding outside the Type-2 blob.
pub fn verify_proposer_signature(
    state: &State,
    block: &Block,
    signature: &[u8],
    backend: &dyn CryptoBackend,
) -> Result<(), TransitionError> {
    if signature.len() != SIGNATURE_BYTES {
        return Err(TransitionError::UnsupportedSignature(format!(
            "proposer signature length {} != {SIGNATURE_BYTES}",
            signature.len()
        )));
    }
    let idx = block.proposer_index.get() as usize;
    let Some(validator) = state.validators.get(idx) else {
        return Err(TransitionError::UnsupportedSignature(format!(
            "proposer index {idx} missing from registry"
        )));
    };
    if validator.index != block.proposer_index {
        return Err(TransitionError::UnsupportedSignature(
            "validator registry index mismatch".into(),
        ));
    }
    let pk = PublicKey::try_from_slice(validator.proposal_public_key.as_bytes())
        .map_err(|e| TransitionError::UnsupportedSignature(e.to_string()))?;
    let sig = Signature::try_from_slice(signature)
        .map_err(|e| TransitionError::UnsupportedSignature(e.to_string()))?;
    let root = block
        .hash_tree_root()
        .map_err(|e| TransitionError::Types(e.to_string()))?;
    verify(backend, &pk, block.slot.get() as u32, &root, &sig)
        .map_err(|e| TransitionError::UnsupportedSignature(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_crypto::{key_gen, sign, TestHmacBackend};
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_types::{
        BlockBody, BlockHeader, Checkpoint, GenesisConfig, Validator,
    };
    use std::sync::Arc;

    fn state_with_proposal_key(pk: PublicKey) -> State {
        let mut pk_bytes = [0u8; 52];
        pk_bytes.copy_from_slice(pk.as_bytes());
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vec![Validator::new(
                Bytes52::ZERO,
                Bytes52(pk_bytes),
                ValidatorIndex::new(0),
            )
            .unwrap()],
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }

    #[test]
    fn accepts_matching_hmac_proposer_sig() {
        let backend = Arc::new(TestHmacBackend::new([0x11; 32]));
        let (pk, sk) = key_gen(backend.as_ref(), 0, 64).unwrap();
        let state = state_with_proposal_key(pk);
        let block = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let root = block.hash_tree_root().unwrap();
        let sig = sign(backend.as_ref(), &sk, 1, &root).unwrap();
        verify_proposer_signature(&state, &block, sig.as_bytes(), backend.as_ref()).unwrap();
    }

    #[test]
    fn rejects_tampered_sig() {
        let backend = Arc::new(TestHmacBackend::new([0x11; 32]));
        let (pk, sk) = key_gen(backend.as_ref(), 0, 64).unwrap();
        let state = state_with_proposal_key(pk);
        let block = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(0),
            parent_root: HASH32_ZERO,
            state_root: [2u8; 32],
            body: BlockBody::default(),
        };
        let root = block.hash_tree_root().unwrap();
        let sig = sign(backend.as_ref(), &sk, 1, &root).unwrap();
        let mut bad = sig.as_bytes().to_vec();
        bad[10] ^= 0xff;
        assert!(verify_proposer_signature(&state, &block, &bad, backend.as_ref()).is_err());
    }
}
