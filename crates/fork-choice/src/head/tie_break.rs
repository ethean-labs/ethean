//! Lexicographic root tie-break helpers.

use ethean_primitives::Hash32;

/// Pick the child with highest `(weight, root)` — larger root wins on a tie.
pub(crate) fn best_child(children: &[Hash32], weights: &std::collections::HashMap<Hash32, u64>) -> Hash32 {
    *children
        .iter()
        .max_by_key(|root| {
            let w = weights.get(*root).copied().unwrap_or(0);
            (w, **root)
        })
        .expect("children non-empty")
}
