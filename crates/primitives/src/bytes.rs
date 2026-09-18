//! Bounded byte containers used by domain and wire layers.

use crate::error::PrimitiveError;

/// XMSS public-key width placeholder (leanSpec Bytes52 / PROD_CONFIG).
pub const XMSS_PUBLIC_KEY_BYTES: usize = 52;

/// Fixed 52-byte container for XMSS public keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bytes52(pub [u8; XMSS_PUBLIC_KEY_BYTES]);

impl Bytes52 {
    /// All-zero key material.
    pub const ZERO: Self = Self([0u8; XMSS_PUBLIC_KEY_BYTES]);

    /// Construct from an exact-length slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, PrimitiveError> {
        if bytes.len() != XMSS_PUBLIC_KEY_BYTES {
            return Err(PrimitiveError::InvalidLength {
                expected: XMSS_PUBLIC_KEY_BYTES,
                got: bytes.len(),
            });
        }
        let mut arr = [0u8; XMSS_PUBLIC_KEY_BYTES];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }

    /// Borrow the inner array.
    pub fn as_bytes(&self) -> &[u8; XMSS_PUBLIC_KEY_BYTES] {
        &self.0
    }
}

impl Default for Bytes52 {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            Bytes52::from_slice(&[0u8; 51]),
            Err(PrimitiveError::InvalidLength {
                expected: 52,
                got: 51,
            })
        );
    }

    #[test]
    fn accepts_exact_length() {
        let key = Bytes52::from_slice(&[7u8; 52]).unwrap();
        assert_eq!(key.as_bytes()[0], 7);
    }

    #[test]
    fn zero_is_zero_width() {
        assert_eq!(Bytes52::ZERO.0.len(), XMSS_PUBLIC_KEY_BYTES);
        assert!(Bytes52::ZERO.0.iter().all(|&b| b == 0));
    }
}
