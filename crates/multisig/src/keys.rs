//! Conversions between Ethean wire types and leanMultisig's XMSS types.

use ethean_crypto::{PublicKey, Signature};
use leansig_wrapper::{
    xmss_public_key_from_ssz, xmss_signature_from_ssz, XmssPublicKey, XmssSignature,
};

use crate::error::{MultisigError, Result};
use crate::limits::MAX_KEYS_PER_COMPONENT;

/// Decode one component's key set, enforcing non-empty and size limits.
pub fn lean_public_keys(component: usize, keys: &[PublicKey]) -> Result<Vec<XmssPublicKey>> {
    if keys.is_empty() {
        return Err(MultisigError::EmptyComponent { component });
    }
    if keys.len() > MAX_KEYS_PER_COMPONENT {
        return Err(MultisigError::LimitExceeded {
            what: "public keys per component",
            actual: keys.len(),
            max: MAX_KEYS_PER_COMPONENT,
        });
    }
    keys.iter()
        .enumerate()
        .map(|(index, key)| {
            xmss_public_key_from_ssz(key.as_bytes())
                .map_err(|()| MultisigError::InvalidPublicKey { index })
        })
        .collect()
}

/// Decode raw `(public key, signature)` pairs for Type-1 aggregation.
pub fn lean_raw_signatures(
    raw: &[(PublicKey, Signature)],
) -> Result<Vec<(XmssPublicKey, XmssSignature)>> {
    raw.iter()
        .enumerate()
        .map(|(index, (key, sig))| {
            let key = xmss_public_key_from_ssz(key.as_bytes())
                .map_err(|()| MultisigError::InvalidPublicKey { index })?;
            let sig = xmss_signature_from_ssz(sig.as_bytes())
                .map_err(|()| MultisigError::InvalidSignature { index })?;
            Ok((key, sig))
        })
        .collect()
}
