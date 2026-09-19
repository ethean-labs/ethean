//! Recovery helpers for prover crashes and partial output.

/// Discard untrusted partial prover output; never resume or cache as valid.
pub fn discard_partial_proof(partial: Vec<u8>) {
    drop(partial);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discard_is_noop_safe() {
        discard_partial_proof(vec![1, 2, 3]);
    }
}
