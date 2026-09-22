//! KoalaBear prime field `p = 2^31 - 2^24 + 1`, canonical `u32` representation.
//!
//! Every value is kept fully reduced in `[0, p)`. Serialization is the
//! leanSpec `Fp` rule: 4 bytes little-endian, non-canonical input rejected.

use crate::error::{CryptoError, Result};

/// KoalaBear prime.
pub const P: u32 = 0x7f00_0001;
const P64: u64 = P as u64;
/// `2^32 mod p`, used for lazy reduction of 64-bit products.
const TWO_POW_32_MOD_P: u64 = (1u64 << 32) - 2 * P64;

/// Field element in canonical form.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Fp(u32);

impl std::fmt::Debug for Fp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Fp {
    pub const ZERO: Fp = Fp(0);
    pub const ONE: Fp = Fp(1);
    /// Largest canonical value, `p - 1`.
    pub const MAX: Fp = Fp(P - 1);

    /// Construct from a canonical integer; `None` if `value >= p`.
    pub const fn new(value: u32) -> Option<Fp> {
        if value < P {
            Some(Fp(value))
        } else {
            None
        }
    }

    /// Construct from a canonical integer, panicking on non-canonical input.
    /// Intended for compile-time constant tables only.
    pub const fn new_const(value: u32) -> Fp {
        assert!(value < P, "non-canonical KoalaBear constant");
        Fp(value)
    }

    /// Reduce an arbitrary `u64` modulo `p`.
    pub const fn from_u64(value: u64) -> Fp {
        Fp((value % P64) as u32)
    }

    /// Reduce an arbitrary `u128` modulo `p`.
    pub const fn from_u128(value: u128) -> Fp {
        Fp((value % (P as u128)) as u32)
    }

    /// Canonical integer value.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Little-endian 4-byte encoding.
    pub const fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Decode 4 little-endian bytes, rejecting non-canonical values.
    pub fn from_le_bytes(bytes: [u8; 4]) -> Result<Fp> {
        Fp::new(u32::from_le_bytes(bytes)).ok_or(CryptoError::NonCanonicalFieldElement)
    }

    #[inline(always)]
    pub const fn add(self, rhs: Fp) -> Fp {
        let sum = self.0 + rhs.0;
        Fp(if sum >= P { sum - P } else { sum })
    }

    #[inline(always)]
    pub const fn sub(self, rhs: Fp) -> Fp {
        let (diff, borrow) = self.0.overflowing_sub(rhs.0);
        Fp(if borrow { diff.wrapping_add(P) } else { diff })
    }

    #[inline(always)]
    pub const fn mul(self, rhs: Fp) -> Fp {
        Fp(((self.0 as u64 * rhs.0 as u64) % P64) as u32)
    }

    #[inline(always)]
    pub const fn square(self) -> Fp {
        self.mul(self)
    }

    /// Poseidon S-box `x^3`.
    #[inline(always)]
    pub const fn cube(self) -> Fp {
        self.square().mul(self)
    }
}

/// Fold a 64-bit product `c * x` (both `< 2^31`) into a `< 2^56` residue.
///
/// `2^32 ≡ 2^25 - 2 (mod p)`, so `hi * 2^32 + lo ≡ hi * (2^25 - 2) + lo`.
/// Summing up to 64 such residues stays below `2^63`, allowing one final
/// reduction per matrix row.
#[inline(always)]
pub const fn lazy_product(c: u32, x: u32) -> u64 {
    let t = c as u64 * x as u64;
    (t & 0xffff_ffff) + (t >> 32) * TWO_POW_32_MOD_P
}

/// Reduce a lazy accumulator produced by [`lazy_product`] sums.
#[inline(always)]
pub const fn reduce_lazy(acc: u64) -> Fp {
    Fp((acc % P64) as u32)
}

/// Decompose a little-endian 256-bit integer into `LIMBS` base-`p` digits,
/// least significant first. Returns `None` if the value does not fit.
pub fn base_p_limbs_from_le_bytes<const LIMBS: usize>(bytes: &[u8; 32]) -> Option<[Fp; LIMBS]> {
    let mut words = [0u64; 4];
    for (i, w) in words.iter_mut().enumerate() {
        *w = u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().expect("8-byte chunk"));
    }
    let mut limbs = [Fp::ZERO; LIMBS];
    for limb in limbs.iter_mut() {
        let mut rem = 0u128;
        for w in words.iter_mut().rev() {
            let cur = (rem << 64) | *w as u128;
            *w = (cur / P as u128) as u64;
            rem = cur % P as u128;
        }
        *limb = Fp(rem as u32);
    }
    if words.iter().any(|&w| w != 0) {
        return None;
    }
    Some(limbs)
}

/// Decompose a `u128` into `LIMBS` base-`p` digits, least significant first.
/// Returns `None` if the value does not fit.
pub const fn base_p_limbs_from_u128<const LIMBS: usize>(mut value: u128) -> Option<[Fp; LIMBS]> {
    let mut limbs = [Fp::ZERO; LIMBS];
    let mut i = 0;
    while i < LIMBS {
        limbs[i] = Fp((value % P as u128) as u32);
        value /= P as u128;
        i += 1;
    }
    if value != 0 {
        return None;
    }
    Some(limbs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_structure() {
        assert_eq!(P, (1u32 << 31) - (1 << 24) + 1);
        assert_eq!(P - 1, 127 * 8u32.pow(8));
        assert_eq!(TWO_POW_32_MOD_P, (1 << 25) - 2);
    }

    #[test]
    fn arithmetic_identities() {
        let a = Fp::new(1_234_567_890).unwrap();
        let b = Fp::MAX;
        assert_eq!(a.add(b), a.sub(Fp::ONE));
        assert_eq!(a.mul(Fp::ONE), a);
        assert_eq!(a.sub(a), Fp::ZERO);
        assert_eq!(b.mul(b), Fp::ONE);
        assert_eq!(a.cube(), a.mul(a).mul(a));
    }

    #[test]
    fn lazy_reduction_matches_direct() {
        let c = P - 5;
        let x = P - 7;
        let direct = Fp(c).mul(Fp(x));
        let mut acc = 0u64;
        for _ in 0..24 {
            acc += lazy_product(c, x);
        }
        let mut expected = Fp::ZERO;
        for _ in 0..24 {
            expected = expected.add(direct);
        }
        assert_eq!(reduce_lazy(acc), expected);
    }

    #[test]
    fn rejects_non_canonical_bytes() {
        assert!(Fp::from_le_bytes(P.to_le_bytes()).is_err());
        assert!(Fp::from_le_bytes((P - 1).to_le_bytes()).is_ok());
    }

    #[test]
    fn base_p_decomposition_known_answers() {
        let mut bytes = [0xffu8; 32];
        bytes[31] = 0x7f;
        let limbs = base_p_limbs_from_le_bytes::<9>(&bytes).unwrap();
        let expected = [
            1835116204, 1695929922, 2050126279, 1837593870, 935597759, 468428768, 1232308909,
            615207528, 136,
        ];
        assert_eq!(limbs.map(Fp::as_u32), expected);
        let mut small = [0u8; 32];
        small[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let limbs = base_p_limbs_from_le_bytes::<9>(&small).unwrap();
        assert_eq!(
            limbs.map(Fp::as_u32),
            [1322555686, 271476955, 0, 0, 0, 0, 0, 0, 0]
        );
        assert!(base_p_limbs_from_le_bytes::<8>(&bytes).is_none());
        assert!(base_p_limbs_from_u128::<2>(1u128 << 62).is_none());
        assert!(base_p_limbs_from_u128::<2>(1u128 << 61).is_some());
    }
}
