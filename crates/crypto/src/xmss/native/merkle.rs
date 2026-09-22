//! Sparse Merkle sub-trees with the leanSpec top/bottom split.
//!
//! Layer `l` holds nodes `[start_index, start_index + len)`; layers are padded
//! to even boundaries with filler digests so every node has a sibling.

use super::params::SchemeParams;
use super::rand::{RandomExt, RandomSource};
use super::tweak::Tweak;
use super::tweak_hash::{hash_pair, Digest, Parameter};
use crate::error::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashTreeLayer {
    pub start_index: u64,
    pub nodes: Vec<Digest>,
}

impl HashTreeLayer {
    /// Pad to an even start and odd end using `filler` for the extra nodes.
    fn padded(
        filler: &mut dyn FnMut() -> Result<Digest>,
        nodes: Vec<Digest>,
        start_index: u64,
    ) -> Result<Self> {
        let end_index = start_index + nodes.len() as u64 - 1;
        let needs_front = start_index & 1 == 1;
        let needs_back = end_index & 1 == 0;
        let mut out = Vec::with_capacity(nodes.len() + 2);
        if needs_front {
            out.push(filler()?);
        }
        out.extend(nodes);
        if needs_back {
            out.push(filler()?);
        }
        Ok(Self {
            start_index: start_index - needs_front as u64,
            nodes: out,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashSubTree {
    pub depth: u64,
    pub lowest_layer: u64,
    pub layers: Vec<HashTreeLayer>,
}

/// Membership proof: siblings from the leaf layer upward.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashTreeOpening {
    pub siblings: Vec<Digest>,
}

fn compute_layer(
    parameter: &Parameter,
    level: u8,
    parent_start: u64,
    children: &[Digest],
) -> Vec<Digest> {
    let pairs: Vec<(usize, &[Digest; 2])> =
        children.as_chunks::<2>().0.iter().enumerate().collect();
    super::parallel::map_parallel(&pairs, |(i, pair)| {
        let tweak = Tweak::Tree {
            level,
            index: (parent_start + *i as u64) as u32,
        };
        hash_pair(parameter, tweak, &pair[0], &pair[1])
    })
}

impl HashSubTree {
    /// Build layers `lowest_layer..=depth` from `nodes` starting at `start_index`.
    pub fn new_subtree(
        filler: &mut dyn FnMut() -> Result<Digest>,
        lowest_layer: usize,
        depth: usize,
        start_index: u64,
        parameter: &Parameter,
        nodes: Vec<Digest>,
    ) -> Result<Self> {
        assert!(lowest_layer < depth, "lowest layer must be below the root");
        assert!(
            start_index + nodes.len() as u64 <= 1u64 << (depth - lowest_layer),
            "nodes do not fit in the layer"
        );
        let mut layers = Vec::with_capacity(depth + 1 - lowest_layer);
        layers.push(HashTreeLayer::padded(filler, nodes, start_index)?);
        for level in lowest_layer..depth {
            let prev = &layers[level - lowest_layer];
            let parent_start = prev.start_index >> 1;
            let parents = compute_layer(parameter, (level + 1) as u8, parent_start, &prev.nodes);
            layers.push(HashTreeLayer::padded(filler, parents, parent_start)?);
        }
        Ok(Self {
            depth: depth as u64,
            lowest_layer: lowest_layer as u64,
            layers,
        })
    }

    /// Top tree: bottom-tree roots as leaves at the half-depth layer.
    pub fn new_top_tree(
        rng: &mut dyn RandomSource,
        params: &SchemeParams,
        start_bottom_index: u64,
        parameter: &Parameter,
        bottom_roots: Vec<Digest>,
    ) -> Result<Self> {
        let depth = params.log_lifetime as usize;
        let mut filler = || rng.digest();
        Self::new_subtree(
            &mut filler,
            depth / 2,
            depth,
            start_bottom_index,
            parameter,
            bottom_roots,
        )
    }

    /// Bottom tree over a full block of leaves; padding never reaches the
    /// wire, so filler digests are zero. The top layer is replaced by the
    /// single root node at index `bottom_index`.
    pub fn new_bottom_tree(
        params: &SchemeParams,
        bottom_index: u64,
        parameter: &Parameter,
        leaves: Vec<Digest>,
    ) -> Result<Self> {
        let depth = params.log_lifetime as usize;
        let half = params.half_depth();
        assert_eq!(leaves.len() as u64, params.leaves_per_bottom_tree());
        let mut filler = || Ok([crate::field::Fp::ZERO; 8]);
        let start_index = bottom_index * params.leaves_per_bottom_tree();
        let mut tree = Self::new_subtree(&mut filler, 0, depth, start_index, parameter, leaves)?;
        let root = tree.layers[half].nodes[(bottom_index % 2) as usize];
        tree.layers.truncate(half);
        tree.layers.push(HashTreeLayer {
            start_index: bottom_index,
            nodes: vec![root],
        });
        Ok(tree)
    }

    pub fn root(&self) -> Digest {
        self.layers.last().expect("tree has a root layer").nodes[0]
    }

    /// Sibling path for `position` up to (excluding) the root layer.
    pub fn path(&self, position: u64) -> HashTreeOpening {
        let first = &self.layers[0];
        assert!(
            position >= first.start_index
                && position < first.start_index + first.nodes.len() as u64,
            "position outside the tree"
        );
        let mut siblings = Vec::with_capacity(self.depth as usize);
        let mut current = position;
        for layer in self
            .layers
            .iter()
            .take((self.depth - self.lowest_layer) as usize)
        {
            if layer.nodes.len() <= 1 {
                break;
            }
            let sibling = (current ^ 1) - layer.start_index;
            siblings.push(layer.nodes[sibling as usize]);
            current >>= 1;
        }
        HashTreeOpening { siblings }
    }
}

/// Bottom path followed by the top path for the bottom tree's root.
pub fn combined_path(top: &HashSubTree, bottom: &HashSubTree, position: u64) -> HashTreeOpening {
    assert_eq!(top.depth, bottom.depth);
    let leaves_per_bottom = 1u64 << (bottom.depth / 2);
    let bottom_index = bottom.layers[0].start_index / leaves_per_bottom;
    let mut siblings = bottom.path(position).siblings;
    siblings.extend(top.path(bottom_index).siblings);
    HashTreeOpening { siblings }
}

/// Recompute the root from a leaf (already hashed) and its opening.
pub fn verify_path(
    parameter: &Parameter,
    root: &Digest,
    position: u64,
    leaf: Digest,
    opening: &HashTreeOpening,
) -> bool {
    let depth = opening.siblings.len();
    if depth > 32 || position >= 1u64 << depth {
        return false;
    }
    let mut node = leaf;
    let mut pos = position;
    for (l, sibling) in opening.siblings.iter().enumerate() {
        let (left, right) = if pos & 1 == 0 {
            (node, *sibling)
        } else {
            (*sibling, node)
        };
        pos >>= 1;
        let tweak = Tweak::Tree {
            level: (l + 1) as u8,
            index: pos as u32,
        };
        node = hash_pair(parameter, tweak, &left, &right);
    }
    node == *root
}
