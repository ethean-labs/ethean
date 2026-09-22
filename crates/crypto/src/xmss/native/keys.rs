//! XMSS key and signature containers with their SSZ codecs.

use super::encoding::Randomness;
use super::merkle::{HashSubTree, HashTreeOpening};
use super::params::{SchemeParams, HASH_LEN, PARAM_LEN, PRF_KEY_LEN, RAND_LEN};
use super::prf::PrfKey;
use super::ssz::{read_digests, read_fps, read_u32, read_u64, write_digests, write_fps};
use super::tweak_hash::{Digest, Parameter};
use crate::error::{CryptoError, Result};

/// Public key: Merkle root plus public parameter (52 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct XmssPublicKey {
    pub root: Digest,
    pub parameter: Parameter,
}

impl XmssPublicKey {
    pub const SSZ_BYTES: usize = (HASH_LEN + PARAM_LEN) * 4;

    pub fn to_ssz(&self) -> [u8; Self::SSZ_BYTES] {
        let mut buf = Vec::with_capacity(Self::SSZ_BYTES);
        write_fps(&mut buf, &self.root);
        write_fps(&mut buf, &self.parameter);
        buf.try_into().expect("fixed public key size")
    }

    pub fn from_ssz(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SSZ_BYTES {
            return Err(CryptoError::InvalidPublicKeyLength {
                expected: Self::SSZ_BYTES,
                got: bytes.len(),
            });
        }
        Ok(Self {
            root: read_fps(&bytes[..HASH_LEN * 4])?,
            parameter: read_fps(&bytes[HASH_LEN * 4..])?,
        })
    }
}

/// Signature: Merkle opening, encoding randomness and released chain digests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XmssSignature {
    pub path: HashTreeOpening,
    pub rho: Randomness,
    pub hashes: Vec<Digest>,
}

impl XmssSignature {
    const FIXED: usize = 4 + RAND_LEN * 4 + 4;

    pub fn to_ssz(&self) -> Vec<u8> {
        let path_len = self.path.ssz_len();
        let mut buf = Vec::with_capacity(Self::FIXED + path_len + self.hashes.len() * 32);
        buf.extend_from_slice(&(Self::FIXED as u32).to_le_bytes());
        write_fps(&mut buf, &self.rho);
        buf.extend_from_slice(&((Self::FIXED + path_len) as u32).to_le_bytes());
        self.path.ssz_append(&mut buf);
        write_digests(&mut buf, &self.hashes);
        buf
    }

    /// Decode and enforce the scheme's opening depth and chain count.
    pub fn from_ssz(params: &SchemeParams, bytes: &[u8]) -> Result<Self> {
        if bytes.len() != params.signature_bytes() {
            return Err(CryptoError::InvalidSignatureLength {
                expected: params.signature_bytes(),
                got: bytes.len(),
            });
        }
        let offset_path = read_u32(bytes, 0)? as usize;
        let rho = read_fps::<RAND_LEN>(&bytes[4..4 + RAND_LEN * 4])?;
        let offset_hashes = read_u32(bytes, 4 + RAND_LEN * 4)? as usize;
        let expected_hashes = Self::FIXED + 4 + params.log_lifetime as usize * 32;
        if offset_path != Self::FIXED || offset_hashes != expected_hashes {
            return Err(CryptoError::MalformedEncoding("signature offsets"));
        }
        let path = HashTreeOpening::from_ssz(&bytes[offset_path..offset_hashes])?;
        let hashes = read_digests(&bytes[offset_hashes..])?;
        if path.siblings.len() != params.log_lifetime as usize || hashes.len() != params.dimension {
            return Err(CryptoError::MalformedEncoding("signature shape"));
        }
        Ok(Self { path, rho, hashes })
    }
}

/// Secret key with the top tree and a two-bottom-tree signing window.
#[derive(Clone)]
pub struct XmssSecretKey {
    pub(crate) prf_key: PrfKey,
    pub(crate) parameter: Parameter,
    pub(crate) activation_epoch: u64,
    pub(crate) num_active_epochs: u64,
    pub(crate) top_tree: HashSubTree,
    pub(crate) left_bottom_tree_index: u64,
    pub(crate) left_bottom_tree: HashSubTree,
    pub(crate) right_bottom_tree: HashSubTree,
}

impl std::fmt::Debug for XmssSecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XmssSecretKey")
            .field("prf_key", &"<redacted>")
            .field("activation_epoch", &self.activation_epoch)
            .field("num_active_epochs", &self.num_active_epochs)
            .field("left_bottom_tree_index", &self.left_bottom_tree_index)
            .finish()
    }
}

impl Drop for XmssSecretKey {
    fn drop(&mut self) {
        self.prf_key = [0u8; PRF_KEY_LEN];
        std::hint::black_box(&self.prf_key);
    }
}

impl XmssSecretKey {
    const FIXED: usize = PRF_KEY_LEN + PARAM_LEN * 4 + 8 + 8 + 4 + 8 + 4 + 4;

    pub fn public_key(&self) -> XmssPublicKey {
        XmssPublicKey {
            root: self.top_tree.root(),
            parameter: self.parameter,
        }
    }

    /// Epochs `[start, end)` the key can ever sign for.
    pub fn activation_interval(&self) -> std::ops::Range<u64> {
        self.activation_epoch..self.activation_epoch + self.num_active_epochs
    }

    /// Epochs currently covered by the two cached bottom trees.
    pub fn prepared_interval(&self, params: &SchemeParams) -> std::ops::Range<u64> {
        let w = params.leaves_per_bottom_tree();
        let start = self.left_bottom_tree_index * w;
        start..start + 2 * w
    }

    pub fn to_ssz(&self) -> Vec<u8> {
        let top = self.top_tree.ssz_len();
        let left = self.left_bottom_tree.ssz_len();
        let right = self.right_bottom_tree.ssz_len();
        let mut buf = Vec::with_capacity(Self::FIXED + top + left + right);
        buf.extend_from_slice(&self.prf_key);
        write_fps(&mut buf, &self.parameter);
        buf.extend_from_slice(&self.activation_epoch.to_le_bytes());
        buf.extend_from_slice(&self.num_active_epochs.to_le_bytes());
        buf.extend_from_slice(&(Self::FIXED as u32).to_le_bytes());
        buf.extend_from_slice(&self.left_bottom_tree_index.to_le_bytes());
        buf.extend_from_slice(&((Self::FIXED + top) as u32).to_le_bytes());
        buf.extend_from_slice(&((Self::FIXED + top + left) as u32).to_le_bytes());
        self.top_tree.ssz_append(&mut buf);
        self.left_bottom_tree.ssz_append(&mut buf);
        self.right_bottom_tree.ssz_append(&mut buf);
        buf
    }

    pub fn from_ssz(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::FIXED {
            return Err(CryptoError::MalformedEncoding("secret key too short"));
        }
        let mut prf_key = [0u8; PRF_KEY_LEN];
        prf_key.copy_from_slice(&bytes[..PRF_KEY_LEN]);
        let mut at = PRF_KEY_LEN;
        let parameter = read_fps::<PARAM_LEN>(&bytes[at..at + PARAM_LEN * 4])?;
        at += PARAM_LEN * 4;
        let activation_epoch = read_u64(bytes, at)?;
        let num_active_epochs = read_u64(bytes, at + 8)?;
        let off_top = read_u32(bytes, at + 16)? as usize;
        let left_bottom_tree_index = read_u64(bytes, at + 20)?;
        let off_left = read_u32(bytes, at + 28)? as usize;
        let off_right = read_u32(bytes, at + 32)? as usize;
        if off_top != Self::FIXED
            || off_left < off_top
            || off_right < off_left
            || off_right > bytes.len()
        {
            return Err(CryptoError::MalformedEncoding("secret key offsets"));
        }
        Ok(Self {
            prf_key,
            parameter,
            activation_epoch,
            num_active_epochs,
            top_tree: HashSubTree::from_ssz(&bytes[off_top..off_left])?,
            left_bottom_tree_index,
            left_bottom_tree: HashSubTree::from_ssz(&bytes[off_left..off_right])?,
            right_bottom_tree: HashSubTree::from_ssz(&bytes[off_right..])?,
        })
    }
}
