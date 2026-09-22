//! Scheme parameters shared by the leanSpec PROD and TEST XMSS configurations.
//!
//! Field-element widths are fixed by the KoalaBear instantiation; only the
//! lifetime, chain count and target sum differ between the two configs.

/// Hash digest length in field elements (`HASH_LENGTH_FIELD_ELEMENTS`).
pub const HASH_LEN: usize = 8;
/// Public parameter length in field elements.
pub const PARAM_LEN: usize = 5;
/// Tweak length in field elements.
pub const TWEAK_LEN: usize = 2;
/// Message length in field elements.
pub const MSG_LEN: usize = 9;
/// Encoding randomness length in field elements.
pub const RAND_LEN: usize = 7;
/// Sponge capacity in field elements.
pub const CAPACITY: usize = 9;
/// Winternitz base `w`.
pub const BASE: u32 = 8;
/// Digits extracted per field element in the aborting message hash.
pub const Z: u32 = 8;
/// Quotient with `Q * BASE^Z == p - 1`.
pub const Q: u32 = 127;
/// Retry bound for the target-sum encoding.
pub const MAX_TRIES: u64 = 100_000;
/// PRF key length in bytes.
pub const PRF_KEY_LEN: usize = 32;
/// Signed message length in bytes.
pub const MESSAGE_LEN: usize = 32;

pub const TWEAK_PREFIX_CHAIN: u8 = 0x00;
pub const TWEAK_PREFIX_TREE: u8 = 0x01;
pub const TWEAK_PREFIX_MESSAGE: u8 = 0x02;

/// Parameters that vary between PROD and TEST.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemeParams {
    /// `log2` of the epoch lifetime; must be even.
    pub log_lifetime: u32,
    /// Number of hash chains `v`.
    pub dimension: usize,
    /// Target sum `T` of the codeword digits.
    pub target_sum: u32,
    /// Human-readable name for logs and fingerprints.
    pub name: &'static str,
}

impl SchemeParams {
    pub const fn lifetime(&self) -> u64 {
        1u64 << self.log_lifetime
    }

    /// Leaves per bottom tree, `2^(log_lifetime / 2)`.
    pub const fn leaves_per_bottom_tree(&self) -> u64 {
        1u64 << (self.log_lifetime / 2)
    }

    /// Half depth, the boundary layer between bottom and top trees.
    pub const fn half_depth(&self) -> usize {
        (self.log_lifetime / 2) as usize
    }

    /// Message-hash output length in field elements, `ceil(v / Z)`.
    pub const fn message_hash_len(&self) -> usize {
        self.dimension.div_ceil(Z as usize)
    }

    /// Chain length `w`; digits range over `0..w`.
    pub const fn chain_length(&self) -> usize {
        BASE as usize
    }

    /// SSZ public key size: root plus parameter.
    pub const fn public_key_bytes(&self) -> usize {
        (HASH_LEN + PARAM_LEN) * 4
    }

    /// SSZ signature size: offsets, rho, path opening and released hashes.
    pub const fn signature_bytes(&self) -> usize {
        4 + RAND_LEN * 4
            + 4
            + 4
            + self.log_lifetime as usize * HASH_LEN * 4
            + self.dimension * HASH_LEN * 4
    }

    /// Constant-check the parameter set against field structure.
    pub const fn validate(&self) {
        assert!(
            self.log_lifetime.is_multiple_of(2),
            "LOG_LIFETIME must be even"
        );
        assert!(self.log_lifetime >= 2 && self.log_lifetime <= 32);
        assert!(self.dimension >= 1 && self.dimension <= 256);
        assert!(self.target_sum <= (self.dimension as u32) * (BASE - 1));
        assert!(self.message_hash_len() <= HASH_LEN);
        assert!((Q as u64) * (BASE as u64).pow(Z) == (crate::field::P as u64) - 1);
    }
}

/// leanSpec `PROD_CONFIG`: lifetime `2^32`, 46 chains, target sum 200.
pub const PROD: SchemeParams = SchemeParams {
    log_lifetime: 32,
    dimension: 46,
    target_sum: 200,
    name: "leanSpec-xmss-prod-l32-d46-b8",
};

/// leanSpec `TEST_CONFIG`: lifetime `2^8`, 4 chains, target sum 6.
///
/// Test-only; never used for consensus.
pub const TEST: SchemeParams = SchemeParams {
    log_lifetime: 8,
    dimension: 4,
    target_sum: 6,
    name: "leanSpec-xmss-test-l8-d4-b8",
};

const _: () = {
    PROD.validate();
    TEST.validate();
    assert!(PROD.public_key_bytes() == 52);
    assert!(PROD.signature_bytes() == 2536);
    assert!(TEST.signature_bytes() == 424);
    assert!(PROD.message_hash_len() == 6);
    assert!(TEST.message_hash_len() == 1);
};
