//! Circulant MDS matrices for the KoalaBear Poseidon1 instances.
//!
//! Row `i` of the matrix is the first row rotated right by `i`, so
//! `M[i][j] = row[(j - i) mod n]` and `y[i] = sum_j M[i][j] * x[j]`.

use crate::field::{lazy_product, reduce_lazy, Fp};

/// First row of the width-16 MDS matrix (small integers).
pub const MDS_ROW_16: [u32; 16] = [1, 1, 51, 1, 11, 17, 2, 1, 101, 63, 15, 2, 67, 22, 13, 3];

/// First row of the width-24 MDS matrix.
pub const MDS_ROW_24: [u32; 24] = [
    0x2D0AAAAB, 0x64850517, 0x17F5551D, 0x04ECBEB5, 0x6D91A8D5, 0x60703026, 0x18D6F3CA, 0x729601A7,
    0x77CDA9E2, 0x3C0F5038, 0x26D52A61, 0x0360405D, 0x68FC71C8, 0x2495A71D, 0x5D57AFC2, 0x1689DD98,
    0x3C2C3DBE, 0x0C23DC41, 0x0524C7F2, 0x6BE4DF69, 0x0A6E572C, 0x5C7790FA, 0x17E118F6, 0x0878A07F,
];

/// Multiply the state by the circulant matrix defined by `row`.
///
/// `y[i] = sum_k row[k] * x[(i + k) mod W]`. Products are folded lazily and
/// reduced once per output element.
#[inline]
pub fn circulant_multiply<const W: usize>(state: &mut [Fp; W], row: &[u32; W]) {
    // Duplicate the input so every output is a plain sliding dot product,
    // which keeps the inner loop branch-free and vectorizable.
    let mut doubled = [0u32; 64];
    for i in 0..W {
        let v = state[i].as_u32();
        doubled[i] = v;
        doubled[i + W] = v;
    }
    for i in 0..W {
        let window = &doubled[i..i + W];
        let mut acc = 0u64;
        for k in 0..W {
            acc += lazy_product(row[k], window[k]);
        }
        state[i] = reduce_lazy(acc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::P;

    #[test]
    fn circulant_matches_schoolbook() {
        let mut state: [Fp; 24] = std::array::from_fn(|i| Fp::from_u64(i as u64 * 987_654_321));
        let input = state;
        circulant_multiply(&mut state, &MDS_ROW_24);
        for i in 0..24 {
            let mut acc = Fp::ZERO;
            for j in 0..24 {
                let coeff = Fp::new(MDS_ROW_24[(j + 24 - i) % 24]).unwrap();
                acc = acc.add(coeff.mul(input[j]));
            }
            assert_eq!(state[i], acc);
        }
    }

    #[test]
    fn rows_are_canonical() {
        assert!(MDS_ROW_16.iter().chain(MDS_ROW_24.iter()).all(|&c| c < P));
    }
}
