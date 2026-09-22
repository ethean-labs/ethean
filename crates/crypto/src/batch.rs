//! Parallel batch verification and a decoded public-key cache.
//!
//! Gossip verifies many independent XMSS signatures per slot; decoding a
//! 52-byte key once per validator and fanning verification across cores is
//! the cheapest win available before aggregation proofs land.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::Result;
use crate::signature::Signature;
use crate::xmss::config::MESSAGE_BYTES;
use crate::xmss::native::{self, parallel, XmssPublicKey, XmssSignature, PROD};

/// One `(public key, epoch, message, signature)` tuple to verify.
pub struct BatchVerifyItem<'a> {
    pub public_key: &'a XmssPublicKey,
    pub epoch: u32,
    pub message: &'a [u8; MESSAGE_BYTES],
    pub signature: &'a Signature,
}

/// Verify every item independently on all available cores. The result
/// vector is index-aligned with `items`; malformed signatures are `false`.
pub fn verify_batch(items: &[BatchVerifyItem<'_>]) -> Vec<bool> {
    parallel::map_parallel(items, |item| {
        match XmssSignature::from_ssz(&PROD, item.signature.as_bytes()) {
            Ok(sig) => native::verify(&PROD, item.public_key, item.epoch, item.message, &sig),
            Err(_) => false,
        }
    })
}

/// Decoded public keys indexed by validator index.
#[derive(Default)]
pub struct PublicKeyCache {
    keys: RwLock<HashMap<u64, Arc<XmssPublicKey>>>,
}

impl PublicKeyCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the decoded key for `index`, decoding and caching `bytes` on a
    /// miss. A key whose bytes change (new registry) is replaced.
    pub fn get_or_decode(&self, index: u64, bytes: &[u8]) -> Result<Arc<XmssPublicKey>> {
        if let Some(existing) = self.keys.read().expect("cache lock").get(&index) {
            if existing.to_ssz()[..] == *bytes {
                return Ok(existing.clone());
            }
        }
        let decoded = Arc::new(XmssPublicKey::from_ssz(bytes)?);
        self.keys
            .write()
            .expect("cache lock")
            .insert(index, decoded.clone());
        Ok(decoded)
    }

    pub fn len(&self) -> usize {
        self.keys.read().expect("cache lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) {
        self.keys.write().expect("cache lock").clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::PublicKey;
    use crate::xmss::config::SIGNATURE_BYTES;

    #[test]
    fn batch_of_garbage_is_all_false_and_aligned() {
        let pk = XmssPublicKey::from_ssz(&[0u8; 52]).unwrap();
        let sig = Signature::from_bytes([0u8; SIGNATURE_BYTES]);
        let msg = [0u8; 32];
        let items: Vec<BatchVerifyItem> = (0..5)
            .map(|i| BatchVerifyItem {
                public_key: &pk,
                epoch: i,
                message: &msg,
                signature: &sig,
            })
            .collect();
        assert_eq!(verify_batch(&items), vec![false; 5]);
    }

    #[test]
    fn cache_decodes_once_and_replaces_on_change() {
        let cache = PublicKeyCache::new();
        let a = PublicKey::from_bytes([0u8; 52]);
        let first = cache.get_or_decode(3, a.as_bytes()).unwrap();
        let again = cache.get_or_decode(3, a.as_bytes()).unwrap();
        assert!(Arc::ptr_eq(&first, &again));
        let mut b = [0u8; 52];
        b[0] = 1;
        let replaced = cache.get_or_decode(3, &b).unwrap();
        assert!(!Arc::ptr_eq(&first, &replaced));
        assert_eq!(cache.len(), 1);
        assert!(cache.get_or_decode(4, &[0u8; 51]).is_err());
    }
}
