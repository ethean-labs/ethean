//! Fixed-width 32-byte digests.

/// Canonical 32-byte hash / root value.
pub type Hash32 = [u8; 32];

/// All-zero hash.
pub const HASH32_ZERO: Hash32 = [0u8; 32];

/// Return true if every byte is zero.
pub fn is_zero(hash: &Hash32) -> bool {
    *hash == HASH32_ZERO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_constant_is_zero() {
        assert!(is_zero(&HASH32_ZERO));
        assert!(!is_zero(&[1u8; 32]));
    }
}
