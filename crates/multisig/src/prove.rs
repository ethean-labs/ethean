//! Proof generation. Runs only inside the `ethean-prover` process.
//!
//! `setup_prover` disables glibc heap trimming and large-block mmap for the
//! whole process and precomputes multi-hundred-MiB DFT tables; leanVM also
//! forbids concurrent proofs in one process. Isolating proving in its own
//! process keeps the node's memory profile and latency independent of it.

use std::sync::Once;

use ethean_crypto::{PublicKey, Signature};
use lean_multisig::{
    aggregate_single_message_signatures, merge_single_message_aggregates,
    split_multi_message_aggregate, MultiMessageAggregateSignature, SingleMessageAggregateSignature,
};

use crate::error::{MultisigError, Result};
use crate::isolate::guarded;
use crate::keys::{lean_public_keys, lean_raw_signatures};
use crate::limits::{
    check_decompressed_size, check_proof_len, slot_to_epoch, LOG_INV_RATE, MAX_CHILDREN,
    MAX_COMPONENTS, MAX_PROOF_BYTES,
};

static PROVER_SETUP: Once = Once::new();

/// A previously produced Type-1 proof together with the keys it covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedProof {
    pub public_keys: Vec<PublicKey>,
    pub proof: Vec<u8>,
}

/// One-time prover initialisation. Call only in a dedicated prover process.
pub fn init_prover_process() {
    PROVER_SETUP.call_once(lean_multisig::setup_prover);
}

fn decode_type1(index: usize, child: &KeyedProof) -> Result<SingleMessageAggregateSignature> {
    check_proof_len(&child.proof)?;
    check_decompressed_size(&child.proof)?;
    let keys = lean_public_keys(index, &child.public_keys)?;
    SingleMessageAggregateSignature::decompress_without_pubkeys(&child.proof, keys)
        .ok_or(MultisigError::Malformed)
}

fn finish(bytes: Vec<u8>) -> Result<Vec<u8>> {
    if bytes.len() > MAX_PROOF_BYTES {
        return Err(MultisigError::ProofLength {
            len: bytes.len(),
            max: MAX_PROOF_BYTES,
        });
    }
    Ok(bytes)
}

/// Aggregate raw signatures and child Type-1 proofs over one `(message, slot)`.
///
/// Children are re-verified by the prover. Needs at least one raw signature or
/// two children (a lone child is already a valid Type-1).
pub fn aggregate_type1(
    children: &[KeyedProof],
    raw: &[(PublicKey, Signature)],
    message: &[u8; 32],
    slot: u64,
) -> Result<Vec<u8>> {
    if raw.is_empty() && children.len() < 2 {
        return Err(MultisigError::ProverFailed(
            "need one raw signature or at least two children".into(),
        ));
    }
    if children.len() > MAX_CHILDREN {
        return Err(MultisigError::LimitExceeded {
            what: "aggregation children",
            actual: children.len(),
            max: MAX_CHILDREN,
        });
    }
    let epoch = slot_to_epoch(slot)?;
    let raw = lean_raw_signatures(raw)?;
    let children = children
        .iter()
        .enumerate()
        .map(|(i, c)| decode_type1(i, c))
        .collect::<Result<Vec<_>>>()?;
    guarded(move || {
        init_prover_process();
        let proof =
            aggregate_single_message_signatures(&children, raw, *message, epoch, LOG_INV_RATE)
                .map_err(|e| MultisigError::ProverFailed(e.to_string()))?;
        finish(proof.compress_without_pubkeys())
    })
}

/// Merge ordered Type-1 proofs (attestations then proposer) into one Type-2.
pub fn merge_type2(components: &[KeyedProof]) -> Result<Vec<u8>> {
    if components.is_empty() || components.len() > MAX_COMPONENTS {
        return Err(MultisigError::LimitExceeded {
            what: "proof components",
            actual: components.len(),
            max: MAX_COMPONENTS,
        });
    }
    let type1s = components
        .iter()
        .enumerate()
        .map(|(i, c)| decode_type1(i, c))
        .collect::<Result<Vec<_>>>()?;
    guarded(move || {
        init_prover_process();
        let merged = merge_single_message_aggregates(type1s, LOG_INV_RATE)
            .map_err(|e| MultisigError::ProverFailed(e.to_string()))?;
        finish(merged.compress_without_pubkeys())
    })
}

/// Re-derive a standalone Type-1 for the component signed over `message`.
pub fn split_type2(
    proof: &[u8],
    public_keys_per_component: &[Vec<PublicKey>],
    message: &[u8; 32],
) -> Result<Vec<u8>> {
    check_proof_len(proof)?;
    check_decompressed_size(proof)?;
    if public_keys_per_component.len() > MAX_COMPONENTS {
        return Err(MultisigError::LimitExceeded {
            what: "proof components",
            actual: public_keys_per_component.len(),
            max: MAX_COMPONENTS,
        });
    }
    let keys = public_keys_per_component
        .iter()
        .enumerate()
        .map(|(i, k)| lean_public_keys(i, k))
        .collect::<Result<Vec<_>>>()?;
    guarded(move || {
        init_prover_process();
        let merged = MultiMessageAggregateSignature::decompress_without_pubkeys(proof, keys)
            .ok_or(MultisigError::Malformed)?;
        let matches: Vec<usize> = merged
            .info
            .iter()
            .enumerate()
            .filter(|(_, info)| info.without_pubkeys.message == *message)
            .map(|(i, _)| i)
            .collect();
        let [index] = matches[..] else {
            return Err(MultisigError::ProverFailed(format!(
                "message matches {} components, expected exactly one",
                matches.len()
            )));
        };
        let single = split_multi_message_aggregate(merged, index, LOG_INV_RATE)
            .map_err(|e| MultisigError::ProverFailed(e.to_string()))?;
        finish(single.compress_without_pubkeys())
    })
}
