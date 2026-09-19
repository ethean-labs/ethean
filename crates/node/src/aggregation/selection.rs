//! Deterministic coverage selection for aggregation.

/// Selection policy for which raw / child proofs enter a merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPolicy {
    /// Prefer higher coverage when ties on message root.
    pub prefer_higher_coverage: bool,
}

impl Default for SelectionPolicy {
    fn default() -> Self {
        Self {
            prefer_higher_coverage: true,
        }
    }
}

/// Pick indices into `coverages` that form a deterministic merge set.
///
/// Current policy: sort by coverage descending, then by original index ascending,
/// take up to `max_take`. Does not invent votes.
pub fn select_coverage(coverages: &[u32], max_take: usize, policy: SelectionPolicy) -> Vec<usize> {
    let mut order: Vec<usize> = (0..coverages.len()).collect();
    if policy.prefer_higher_coverage {
        order.sort_by(|&a, &b| {
            coverages[b]
                .cmp(&coverages[a])
                .then_with(|| a.cmp(&b))
        });
    } else {
        order.sort();
    }
    order.into_iter().take(max_take).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_tie_break() {
        let sel = select_coverage(&[3, 5, 5, 1], 2, SelectionPolicy::default());
        assert_eq!(sel, vec![1, 2]);
    }
}
