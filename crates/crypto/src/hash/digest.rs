//! Domain-separated SHA-256 digests.
//!
//! These helpers are not Poseidon and must not be described as production XMSS hashes.

use sha2::{Digest, Sha256};

use crate::domain;

/// 32-byte digest.
pub type Digest32 = [u8; 32];

/// Domain-separated SHA-256 over `domain || data`.
pub fn domain_digest(domain: &[u8], data: &[u8]) -> Digest32 {
    let mut hasher = Sha256::new();
    hasher.update((domain.len() as u32).to_le_bytes());
    hasher.update(domain);
    hasher.update((data.len() as u32).to_le_bytes());
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash a 32-byte signing root for journal / metrics (never log the root itself as a label).
pub fn signing_root_digest(root: &[u8; 32]) -> Digest32 {
    domain_digest(domain::DOMAIN_SIGNING_ROOT, root)
}

/// Hash serialized signature bytes for durable completion records.
pub fn signature_hash(sig_bytes: &[u8]) -> Digest32 {
    domain_digest(domain::DOMAIN_SIG_HASH, sig_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_separation_changes_digest() {
        let a = domain_digest(b"a", b"payload");
        let b = domain_digest(b"b", b"payload");
        assert_ne!(a, b);
    }
}
