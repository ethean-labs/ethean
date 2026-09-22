//! Leaf computation: chain starts from the PRF, full chain walks, sponge leaf.

use super::merkle::HashSubTree;
use super::params::SchemeParams;
use super::prf::{chain_start, PrfKey};
use super::tweak_hash::{chain, hash_leaf, leaf_capacity, Digest, Parameter};
use crate::error::Result;

/// Leaf digest for one epoch: hash of all `dimension` chain ends.
pub fn epoch_leaf(
    params: &SchemeParams,
    prf_key: &PrfKey,
    parameter: &Parameter,
    capacity: &[crate::field::Fp; super::params::CAPACITY],
    epoch: u32,
) -> Digest {
    let steps = params.chain_length() - 1;
    let ends: Vec<Digest> = (0..params.dimension)
        .map(|i| {
            let start = chain_start(prf_key, epoch, i as u64);
            chain(parameter, epoch, i as u8, 0, steps, &start)
        })
        .collect();
    hash_leaf(parameter, capacity, epoch, &ends)
}

/// All leaves of bottom tree `bottom_index`, computed in parallel.
pub fn bottom_tree_leaves(
    params: &SchemeParams,
    prf_key: &PrfKey,
    parameter: &Parameter,
    bottom_index: u64,
) -> Vec<Digest> {
    let capacity = leaf_capacity(params);
    let w = params.leaves_per_bottom_tree();
    let epochs: Vec<u32> = (bottom_index * w..(bottom_index + 1) * w)
        .map(|e| e as u32)
        .collect();
    super::parallel::map_parallel(&epochs, |&epoch| {
        epoch_leaf(params, prf_key, parameter, &capacity, epoch)
    })
}

/// Build bottom tree `bottom_index` from the PRF key.
pub fn bottom_tree_from_prf(
    params: &SchemeParams,
    prf_key: &PrfKey,
    parameter: &Parameter,
    bottom_index: u64,
) -> Result<HashSubTree> {
    let leaves = bottom_tree_leaves(params, prf_key, parameter, bottom_index);
    HashSubTree::new_bottom_tree(params, bottom_index, parameter, leaves)
}
