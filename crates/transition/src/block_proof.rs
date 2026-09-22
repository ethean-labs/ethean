//! Block proof verification (leanSpec `lstar/signatures.py::verify_signatures`).
//!
//! A block carries no individual signatures, only one merged Type-2 proof.
//! Its components are, in order: one per body attestation (the voters'
//! attestation keys over `hash_tree_root(data)` at `data.slot`), then the
//! proposer's proposal key over `hash_tree_root(block)` at `block.slot`.

use ethean_crypto::{AggregateVerifier, ProofComponent, PublicKey};
use ethean_primitives::Bytes52;
use ethean_types::{Block, SignedBlock, Validator};

use crate::error::TransitionError;

fn public_key(bytes: &Bytes52) -> Result<PublicKey, TransitionError> {
    PublicKey::try_from_slice(bytes.as_bytes())
        .map_err(|e| TransitionError::InvalidBlockProof(e.to_string()))
}

/// Indices of the set bits of an aggregation bitfield.
pub fn participant_indices(bits: &[bool]) -> Vec<u64> {
    bits.iter()
        .enumerate()
        .filter_map(|(i, set)| set.then_some(i as u64))
        .collect()
}

/// Rebuild the ordered components a block proof must bind, from the parent
/// state's registry. Out-of-range voter or proposer indices are rejected
/// before indexing, as the spec requires.
pub fn block_proof_components(
    block: &Block,
    validators: &[Validator],
) -> Result<Vec<ProofComponent>, TransitionError> {
    let registry = validators.len() as u64;
    let mut components = Vec::with_capacity(block.body.attestations.len() + 1);
    for attestation in &block.body.attestations {
        let voters = participant_indices(&attestation.aggregation_bits.bits);
        let mut public_keys = Vec::with_capacity(voters.len());
        for voter in voters {
            if voter >= registry {
                return Err(TransitionError::ValidatorIndexOutOfRange(format!(
                    "attester {voter} outside registry of {registry}"
                )));
            }
            public_keys.push(public_key(
                &validators[voter as usize].attestation_public_key,
            )?);
        }
        components.push(ProofComponent {
            public_keys,
            message: attestation.data.hash_tree_root(),
            slot: attestation.data.slot.get(),
        });
    }
    let proposer = block.proposer_index.get();
    if proposer >= registry {
        return Err(TransitionError::ProposerIndexOutOfRange(format!(
            "proposer {proposer} outside registry of {registry}"
        )));
    }
    let block_root = block
        .hash_tree_root()
        .map_err(|e| TransitionError::Types(e.to_string()))?;
    components.push(ProofComponent {
        public_keys: vec![public_key(
            &validators[proposer as usize].proposal_public_key,
        )?],
        message: block_root,
        slot: block.slot.get(),
    });
    Ok(components)
}

/// Verify the merged proof of `signed` against the parent state's registry.
pub fn verify_block_proof(
    signed: &SignedBlock,
    validators: &[Validator],
    verifier: &dyn AggregateVerifier,
) -> Result<(), TransitionError> {
    let components = block_proof_components(&signed.block, validators)?;
    verifier
        .verify_multi(&signed.proof.proof, &components)
        .map_err(|e| TransitionError::InvalidBlockProof(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_crypto::{CryptoError, Result};
    use ethean_primitives::{Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_types::{
        AggregatedAttestation, AggregationBits, AttestationData, BlockBody, Checkpoint,
        MultiMessageAggregate,
    };
    use std::sync::Mutex;

    #[derive(Default)]
    struct Recording(Mutex<Vec<ProofComponent>>);

    impl AggregateVerifier for Recording {
        fn verify_single(&self, _: &[u8], _: &[PublicKey], _: &[u8; 32], _: u64) -> Result<()> {
            Err(CryptoError::VerificationFailed)
        }
        fn verify_multi(&self, _: &[u8], components: &[ProofComponent]) -> Result<()> {
            *self.0.lock().unwrap() = components.to_vec();
            Ok(())
        }
    }

    fn registry(n: u8) -> Vec<Validator> {
        (0..n)
            .map(|i| {
                Validator::new(
                    Bytes52([i; 52]),
                    Bytes52([0x80 | i; 52]),
                    ValidatorIndex::new(i as u64),
                )
                .unwrap()
            })
            .collect()
    }

    fn vote(slot: u64, bits: Vec<bool>) -> AggregatedAttestation {
        AggregatedAttestation {
            aggregation_bits: AggregationBits { bits },
            data: AttestationData {
                slot: Slot::new(slot),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        }
    }

    fn signed(proposer: u64, attestations: Vec<AggregatedAttestation>) -> SignedBlock {
        SignedBlock {
            block: Block {
                slot: Slot::new(9),
                proposer_index: ValidatorIndex::new(proposer),
                parent_root: HASH32_ZERO,
                state_root: HASH32_ZERO,
                body: BlockBody::new(attestations).unwrap(),
            },
            proof: MultiMessageAggregate::new(vec![1, 2, 3]).unwrap(),
        }
    }

    #[test]
    fn components_follow_spec_order_keys_and_bindings() {
        let validators = registry(4);
        let a = vote(7, vec![false, true, false, true]);
        let b = vote(8, vec![true]);
        let block = signed(2, vec![a.clone(), b.clone()]);
        let recorder = Recording::default();
        verify_block_proof(&block, &validators, &recorder).unwrap();
        let got = recorder.0.lock().unwrap().clone();
        assert_eq!(got.len(), 3);
        assert_eq!(
            got[0].public_keys,
            vec![
                PublicKey::from_bytes([1; 52]),
                PublicKey::from_bytes([3; 52])
            ]
        );
        assert_eq!(got[0].message, a.data.hash_tree_root());
        assert_eq!(got[0].slot, 7, "attestation binds its own slot");
        assert_eq!(got[1].public_keys, vec![PublicKey::from_bytes([0; 52])]);
        assert_eq!(got[1].slot, 8);
        assert_eq!(got[2].public_keys, vec![PublicKey::from_bytes([0x82; 52])]);
        assert_eq!(got[2].message, block.block.hash_tree_root().unwrap());
        assert_eq!(got[2].slot, 9);
    }

    #[test]
    fn rejects_out_of_range_indices_before_verifying() {
        let validators = registry(2);
        let voter_oob = signed(0, vec![vote(1, vec![false, false, true])]);
        assert!(matches!(
            verify_block_proof(&voter_oob, &validators, &Recording::default()),
            Err(TransitionError::ValidatorIndexOutOfRange(_))
        ));
        let proposer_oob = signed(2, vec![]);
        assert!(matches!(
            verify_block_proof(&proposer_oob, &validators, &Recording::default()),
            Err(TransitionError::ProposerIndexOutOfRange(_))
        ));
    }
}
