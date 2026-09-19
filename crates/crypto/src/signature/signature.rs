//! Strict 2536-byte XMSS signature (leanSpec PROD_CONFIG).

use crate::error::{CryptoError, Result};
use crate::xmss::config::SIGNATURE_BYTES;

/// Wire signature: exactly [`SIGNATURE_BYTES`] bytes.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    bytes: Box<[u8; SIGNATURE_BYTES]>,
}

impl Signature {
    /// Construct from an exact-length byte array.
    pub fn from_bytes(bytes: [u8; SIGNATURE_BYTES]) -> Self {
        Self {
            bytes: Box::new(bytes),
        }
    }

    /// Construct from a slice; rejects wrong lengths.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != SIGNATURE_BYTES {
            return Err(CryptoError::InvalidSignatureLength {
                expected: SIGNATURE_BYTES,
                got: bytes.len(),
            });
        }
        let mut arr = [0u8; SIGNATURE_BYTES];
        arr.copy_from_slice(bytes);
        Ok(Self {
            bytes: Box::new(arr),
        })
    }

    /// Borrow raw bytes.
    pub fn as_bytes(&self) -> &[u8; SIGNATURE_BYTES] {
        &self.bytes
    }

    /// Copy into a heap array (signatures are large).
    pub fn to_bytes(&self) -> Box<[u8; SIGNATURE_BYTES]> {
        self.bytes.clone()
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature(len={})", SIGNATURE_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_length() {
        assert!(matches!(
            Signature::try_from_slice(&[0u8; 10]),
            Err(CryptoError::InvalidSignatureLength {
                expected: 2536,
                got: 10
            })
        ));
    }

    #[test]
    fn accepts_exact_length() {
        let sig = Signature::try_from_slice(&[1u8; SIGNATURE_BYTES]).unwrap();
        assert_eq!(sig.as_bytes()[0], 1);
        assert_eq!(sig.as_bytes().len(), 2536);
    }
}
