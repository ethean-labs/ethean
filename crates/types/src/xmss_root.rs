//! Hash tree root of an XMSS signature laid out as the leanSpec container
//! `Signature { path: HashTreeOpening { siblings }, rho, hashes }`.
//!
//! Ethean keeps signatures as opaque bytes; only the root needs the layout.

use ethean_ssz::{chunk_from_bytes, hash_tree_root_container, hash_tree_root_list, Root};

use crate::error::TypesError;

/// `NODE_LIST_LIMIT = 2 * LEAVES_PER_BOTTOM_TREE` for `LOG_LIFETIME = 32`.
pub const XMSS_NODE_LIST_LIMIT: usize = 2 << 16;
/// `Randomness` is `Vector[Fp, 7]`: 28 bytes.
const RHO_BYTES: usize = 28;
/// `HashDigestVector` is `Vector[Fp, 8]`: one 32-byte chunk.
const DIGEST_BYTES: usize = 32;

fn u32_at(bytes: &[u8], at: usize) -> Result<usize, TypesError> {
    let raw = bytes
        .get(at..at + 4)
        .ok_or_else(|| TypesError::InvalidContainer("signature too short".into()))?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize)
}

fn digest_list_root(bytes: &[u8]) -> Result<Root, TypesError> {
    if bytes.len() % DIGEST_BYTES != 0 {
        return Err(TypesError::InvalidContainer(
            "digest list is not a multiple of 32 bytes".into(),
        ));
    }
    let chunks: Vec<Root> = bytes.chunks(DIGEST_BYTES).map(chunk_from_bytes).collect();
    Ok(hash_tree_root_list(&chunks, XMSS_NODE_LIST_LIMIT)?)
}

/// Root of a serialized XMSS signature.
pub fn xmss_signature_root(sig: &[u8]) -> Result<Root, TypesError> {
    let path_off = u32_at(sig, 0)?;
    let hashes_off = u32_at(sig, 4 + RHO_BYTES)?;
    if path_off != 8 + RHO_BYTES || hashes_off < path_off + 4 || hashes_off > sig.len() {
        return Err(TypesError::InvalidContainer(
            "signature offsets are inconsistent".into(),
        ));
    }
    let rho = &sig[4..4 + RHO_BYTES];
    let path = &sig[path_off..hashes_off];
    if u32_at(path, 0)? != 4 {
        return Err(TypesError::InvalidContainer(
            "signature path offset must be 4".into(),
        ));
    }
    let path_root = digest_list_root(&path[4..])?;
    let hashes_root = digest_list_root(&sig[hashes_off..])?;
    Ok(hash_tree_root_container(&[
        path_root,
        chunk_from_bytes(rho),
        hashes_root,
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_and_misaligned_input() {
        assert!(xmss_signature_root(&[0u8; 10]).is_err());
        let mut sig = vec![0u8; 2536];
        sig[..4].copy_from_slice(&36u32.to_le_bytes());
        sig[32..36].copy_from_slice(&1064u32.to_le_bytes());
        sig[36..40].copy_from_slice(&4u32.to_le_bytes());
        assert!(xmss_signature_root(&sig).is_ok());
        sig[36..40].copy_from_slice(&8u32.to_le_bytes());
        assert!(xmss_signature_root(&sig).is_err());
    }
}
