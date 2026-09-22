//! In-process verification of leanMultisig proofs.
//!
//! Only `setup_verifier` runs here (bytecode compilation). The prover setup,
//! which changes process-wide malloc settings, never runs in the node.

use std::sync::Once;

use ethean_crypto::{AggregateVerifier, CryptoError, ProofComponent, PublicKey};
use lean_multisig::{
    verify_multi_message_aggregate, verify_single_message_aggregate,
    MultiMessageAggregateSignature, SingleMessageAggregateSignature,
};

use crate::error::{MultisigError, Result};
use crate::isolate::guarded;
use crate::keys::lean_public_keys;
use crate::limits::{check_decompressed_size, check_proof_len, slot_to_epoch, MAX_COMPONENTS};

static VERIFIER_SETUP: Once = Once::new();

/// Compile the aggregation bytecode once (on an isolated large-stack thread;
/// the compiler recurses deeply). Idempotent and cheap after the first call.
pub fn init_verifier() {
    VERIFIER_SETUP.call_once(|| {
        let _ = guarded(|| {
            lean_multisig::setup_verifier();
            Ok(())
        });
    });
}

/// Verify a single-message (Type-1) proof for `public_keys` over `(message, slot)`.
pub fn verify_single(
    proof: &[u8],
    public_keys: &[PublicKey],
    message: &[u8; 32],
    slot: u64,
) -> Result<()> {
    check_proof_len(proof)?;
    check_decompressed_size(proof)?;
    let epoch = slot_to_epoch(slot)?;
    let keys = lean_public_keys(0, public_keys)?;
    init_verifier();
    guarded(move || {
        let sig = SingleMessageAggregateSignature::decompress_without_pubkeys(proof, keys)
            .ok_or(MultisigError::Malformed)?;
        let bound = &sig.info.without_pubkeys;
        if bound.message != *message || bound.slot != epoch {
            return Err(MultisigError::BindingMismatch { component: 0 });
        }
        verify_single_message_aggregate(&sig)
            .map(|_| ())
            .map_err(|e| MultisigError::Rejected(format!("{e:?}")))
    })
}

/// Verify a multi-message (Type-2) proof against ordered component bindings.
pub fn verify_multi(proof: &[u8], components: &[ProofComponent]) -> Result<()> {
    check_proof_len(proof)?;
    check_decompressed_size(proof)?;
    if components.is_empty() || components.len() > MAX_COMPONENTS {
        return Err(MultisigError::LimitExceeded {
            what: "proof components",
            actual: components.len(),
            max: MAX_COMPONENTS,
        });
    }
    let mut keys = Vec::with_capacity(components.len());
    let mut bindings = Vec::with_capacity(components.len());
    for (index, component) in components.iter().enumerate() {
        keys.push(lean_public_keys(index, &component.public_keys)?);
        bindings.push((component.message, slot_to_epoch(component.slot)?));
    }
    init_verifier();
    guarded(move || {
        let sig = MultiMessageAggregateSignature::decompress_without_pubkeys(proof, keys)
            .ok_or(MultisigError::Malformed)?;
        if sig.info.len() != bindings.len() {
            return Err(MultisigError::ComponentCount {
                expected: bindings.len(),
                got: sig.info.len(),
            });
        }
        for (component, (info, (message, epoch))) in sig.info.iter().zip(&bindings).enumerate() {
            let bound = &info.without_pubkeys;
            if bound.message != *message || bound.slot != *epoch {
                return Err(MultisigError::BindingMismatch { component });
            }
        }
        verify_multi_message_aggregate(&sig)
            .map(|_| ())
            .map_err(|e| MultisigError::Rejected(format!("{e:?}")))
    })
}

/// [`AggregateVerifier`] backed by leanMultisig at the pq-devnet-4 pin.
#[derive(Debug, Default, Clone, Copy)]
pub struct LeanMultisigVerifier;

impl AggregateVerifier for LeanMultisigVerifier {
    fn verify_single(
        &self,
        proof: &[u8],
        public_keys: &[PublicKey],
        message: &[u8; 32],
        slot: u64,
    ) -> ethean_crypto::Result<()> {
        verify_single(proof, public_keys, message, slot)
            .map_err(|e| CryptoError::InvalidAggregate(e.to_string()))
    }

    fn verify_multi(
        &self,
        proof: &[u8],
        components: &[ProofComponent],
    ) -> ethean_crypto::Result<()> {
        verify_multi(proof, components).map_err(|e| CryptoError::InvalidAggregate(e.to_string()))
    }
}
