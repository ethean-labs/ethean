//! SHA-256 based `hash_tree_root` for basic SSZ types.

use sha2::{Digest, Sha256};

use crate::error::SszError;

/// 32-byte SSZ chunk / root.
pub type Root = [u8; 32];

/// All-zero chunk.
pub const ZERO_CHUNK: Root = [0u8; 32];

/// Hash two 32-byte children (parent merkle node).
pub fn hash_nodes(left: &Root, right: &Root) -> Root {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// Pack little-endian integer bytes into a single 32-byte chunk (zero-padded).
pub fn chunk_from_bytes(bytes: &[u8]) -> Root {
    let mut chunk = ZERO_CHUNK;
    let n = bytes.len().min(32);
    chunk[..n].copy_from_slice(&bytes[..n]);
    chunk
}

/// `hash_tree_root` of a `uint64`.
pub fn hash_tree_root_u64(value: u64) -> Root {
    chunk_from_bytes(&value.to_le_bytes())
}

/// `hash_tree_root` of a boolean.
pub fn hash_tree_root_bool(value: bool) -> Root {
    chunk_from_bytes(&[u8::from(value)])
}

/// `hash_tree_root` of a fixed-size byte array (packed into 32-byte chunks, then merkleized).
pub fn hash_tree_root_bytes(bytes: &[u8]) -> Root {
    if bytes.is_empty() {
        return ZERO_CHUNK;
    }
    let mut chunks = Vec::new();
    for chunk in bytes.chunks(32) {
        chunks.push(chunk_from_bytes(chunk));
    }
    merkleize(&chunks, next_power_of_two(chunks.len()))
}

/// Root of an all-zero merkle tree with exactly `leaf_count` leaves (`leaf_count` is a power of two).
fn zero_merkle_root(leaf_count: usize) -> Root {
    debug_assert!(leaf_count.is_power_of_two() && leaf_count > 0);
    let mut h = ZERO_CHUNK;
    let mut n = 1usize;
    while n < leaf_count {
        h = hash_nodes(&h, &h);
        n *= 2;
    }
    h
}

/// Merkleize `chunks`, padding with zero hashes up to `limit` (power of two).
///
/// Uses virtual zero-subtree padding so large SSZ limits (e.g. justification bitlists)
/// do not allocate `limit` chunks.
pub fn merkleize(chunks: &[Root], limit: usize) -> Root {
    debug_assert!(limit.is_power_of_two() || limit == 0);
    if limit == 0 {
        return ZERO_CHUNK;
    }
    if chunks.is_empty() {
        return zero_merkle_root(limit);
    }

    let mut width = chunks.len().next_power_of_two().max(1);
    let mut layer = chunks.to_vec();
    layer.resize(width, ZERO_CHUNK);

    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len() / 2);
        for pair in layer.chunks(2) {
            next.push(hash_nodes(&pair[0], &pair[1]));
        }
        layer = next;
    }
    let mut root = layer[0];

    while width < limit {
        let zero_side = zero_merkle_root(width);
        root = hash_nodes(&root, &zero_side);
        width = width.saturating_mul(2);
    }
    root
}

/// Mix a merkleized list root with its length (SSZ list / bitlist length mixin).
pub fn mix_in_length(root: &Root, length: u64) -> Root {
    let len_chunk = hash_tree_root_u64(length);
    hash_nodes(root, &len_chunk)
}

/// `hash_tree_root` of a list of already-computed element roots with SSZ limit `limit`.
pub fn hash_tree_root_list(element_roots: &[Root], limit: usize) -> Result<Root, SszError> {
    if element_roots.len() > limit {
        return Err(SszError::ListTooLong {
            got: element_roots.len(),
            limit,
        });
    }
    let chunk_count = next_power_of_two(limit.max(1));
    let tree = merkleize(element_roots, chunk_count);
    Ok(mix_in_length(&tree, element_roots.len() as u64))
}

/// `hash_tree_root` of a container given ordered field roots.
pub fn hash_tree_root_container(field_roots: &[Root]) -> Root {
    if field_roots.is_empty() {
        return ZERO_CHUNK;
    }
    let limit = next_power_of_two(field_roots.len());
    merkleize(field_roots, limit)
}

/// Bitlist root: pack bits into chunks, merkleize to limit chunks, mix in bit length.
pub fn hash_tree_root_bitlist(bits: &[bool], limit: usize) -> Result<Root, SszError> {
    if bits.len() > limit {
        return Err(SszError::ListTooLong {
            got: bits.len(),
            limit,
        });
    }
    let mut packed = vec![0u8; (bits.len() + 7) / 8];
    for (i, &b) in bits.iter().enumerate() {
        if b {
            packed[i / 8] |= 1 << (i % 8);
        }
    }
    let chunks: Vec<Root> = if packed.is_empty() {
        Vec::new()
    } else {
        packed.chunks(32).map(chunk_from_bytes).collect()
    };
    let chunk_limit = next_power_of_two(((limit + 255) / 256).max(1));
    let tree = merkleize(&chunks, chunk_limit);
    Ok(mix_in_length(&tree, bits.len() as u64))
}

fn next_power_of_two(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        n.next_power_of_two()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_root_is_padded_le() {
        let root = hash_tree_root_u64(1);
        assert_eq!(root[0], 1);
        assert!(root[1..].iter().all(|&b| b == 0));
    }

    #[test]
    fn bool_roots_differ() {
        assert_ne!(hash_tree_root_bool(false), hash_tree_root_bool(true));
    }

    #[test]
    fn container_two_fields_stable() {
        let a = hash_tree_root_u64(1);
        let b = hash_tree_root_u64(2);
        let r1 = hash_tree_root_container(&[a, b]);
        let r2 = hash_tree_root_container(&[a, b]);
        assert_eq!(r1, r2);
        assert_ne!(r1, hash_tree_root_container(&[b, a]));
    }

    #[test]
    fn bytes32_is_identity_chunk() {
        let bytes = [7u8; 32];
        assert_eq!(hash_tree_root_bytes(&bytes), bytes);
    }

    #[test]
    fn large_empty_bitlist_is_fast() {
        // JustificationValidators limit is ~2^30 bits → millions of leaf chunks if materialised.
        let root = hash_tree_root_bitlist(&[], 1_073_741_824).unwrap();
        assert_ne!(root, ZERO_CHUNK); // length mixin
    }

    #[test]
    fn merkleize_matches_naive_small() {
        let chunks = [hash_tree_root_u64(1), hash_tree_root_u64(2)];
        let efficient = merkleize(&chunks, 8);
        // Manual: pad to 8 then fold
        let mut layer = chunks.to_vec();
        layer.resize(8, ZERO_CHUNK);
        while layer.len() > 1 {
            let mut next = Vec::new();
            for pair in layer.chunks(2) {
                next.push(hash_nodes(&pair[0], &pair[1]));
            }
            layer = next;
        }
        assert_eq!(efficient, layer[0]);
    }
}
