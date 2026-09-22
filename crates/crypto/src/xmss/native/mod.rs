//! Dependency-free leanSpec XMSS: KoalaBear field, Poseidon1 tweakable hash,
//! SHAKE128 PRF, aborting target-sum encoding and top/bottom Merkle trees.

pub mod encoding;
pub mod keys;
pub mod leaves;
pub mod merkle;
pub mod parallel;
pub mod params;
pub mod prf;
pub mod rand;
pub mod scheme;
pub mod ssz;
pub mod tweak;
pub mod tweak_hash;

pub use encoding::{target_sum_encode, Randomness};
pub use keys::{XmssPublicKey, XmssSecretKey, XmssSignature};
pub use merkle::{HashSubTree, HashTreeLayer, HashTreeOpening};
pub use params::{SchemeParams, PROD, TEST};
pub use rand::{OsRandom, RandomExt, RandomSource, SeededRandom};
pub use scheme::{advance_preparation, key_gen, prepare_for_epoch, sign, verify};
pub use tweak::Tweak;
pub use tweak_hash::{Digest, Parameter};

#[cfg(test)]
mod tests;
