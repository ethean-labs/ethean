//! Strict 52-byte XMSS public key (leanSpec PROD_CONFIG / Bytes52).

use crate::error::{CryptoError, Result};
use crate::xmss::config::PUBLIC_KEY_BYTES;

/// Wire public key: exactly [`PUBLIC_KEY_BYTES`] bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey {
    bytes: [u8; PUBLIC_KEY_BYTES],
}

impl PublicKey {
    /// Construct from an exact-length byte array.
    pub const fn from_bytes(bytes: [u8; PUBLIC_KEY_BYTES]) -> Self {
        Self { bytes }
    }

    /// Construct from a slice; rejects wrong lengths.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != PUBLIC_KEY_BYTES {
            return Err(CryptoError::InvalidPublicKeyLength {
                expected: PUBLIC_KEY_BYTES,
                got: bytes.len(),
            });
        }
        let mut arr = [0u8; PUBLIC_KEY_BYTES];
        arr.copy_from_slice(bytes);
        Ok(Self { bytes: arr })
    }

    /// Borrow raw bytes.
    pub const fn as_bytes(&self) -> &[u8; PUBLIC_KEY_BYTES] {
        &self.bytes
    }

    /// Copy raw bytes.
    pub const fn to_bytes(&self) -> [u8; PUBLIC_KEY_BYTES] {
        self.bytes
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey(len={})", PUBLIC_KEY_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_length() {
        assert!(matches!(
            PublicKey::try_from_slice(&[0u8; 51]),
            Err(CryptoError::InvalidPublicKeyLength { expected: 52, got: 51 })
        ));
    }

    #[test]
    fn accepts_exact_length() {
        let pk = PublicKey::try_from_slice(&[9u8; 52]).unwrap();
        assert_eq!(pk.as_bytes()[0], 9);
    }
}
