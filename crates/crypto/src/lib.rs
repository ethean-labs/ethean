//! Ethean Lean Consensus crypto — XMSS wire types, aggregation, backends.
//!
//! XMSS is implemented natively (see [`xmss::native`]) and verified against
//! leanSpec vectors and published leanSig keys. leanVM aggregation paths fail
//! closed unless a prover is wired. Default test helpers verify with real
//! bindings (never always-true).

#![forbid(unsafe_code)]

pub mod aggregation;
pub mod backend;
#[cfg(feature = "leanvm-backend")]
mod backend_leanvm;
#[cfg(any(test, feature = "test-hmac"))]
mod backend_test_hmac;
pub mod batch;
pub mod domain;
pub mod error;
pub mod ffi_status;
pub mod field;
pub mod hash;
pub mod keccak;
pub mod leanvm_ipc;
mod leanvm_ipc_frame;
mod leanvm_ipc_spawn;
mod leanvm_ipc_split;
pub mod poseidon;
pub mod signature;
pub mod xmss;

pub use aggregation::{
    aggregation_fingerprint, assert_aggregation_invariants, attestation_leaves_from_type2,
    merge_type1, merge_type1_statements, prove_type1, prove_type2, split_type2_to_type1,
    verify_statement_shape, verify_type1, verify_type2, AggregateStatement, ParticipantSet,
    ProofKind, Type1Leaf, Type2ComponentRef, LEANVM_REV, LOG_INV_RATE, MAX_PROOF_BYTES,
    PROD_AGGREGATION_FINGERPRINT,
};
#[cfg(any(test, feature = "test-hmac"))]
pub use backend::TestHmacBackend;
pub use backend::{CryptoBackend, ProductionBackend, SecretKeyMaterial};
pub use batch::{verify_batch, BatchVerifyItem, PublicKeyCache};
pub use error::{CryptoError, Result};
pub use ffi_status::{
    BackendGap, FfiStatus, LeanSigGate, LeanVmGate, LEANSIG_VENDOR_BIGINT_PATCH_REQUIRED,
};
pub use field::Fp;
pub use hash::{domain_digest, signature_hash, signing_root_digest, Digest32};
pub use leanvm_ipc::{
    try_roundtrip_prove, LeanVmIpcStatus, PROBE_ENV as LEANVM_IPC_PROBE_ENV,
    PROVER_ENV as LEANVM_PROVER_ENV,
};
pub use leanvm_ipc_frame::{IpcFrame, IpcOp, FRAME_CODEC_READY, FRAME_MAGIC, FRAME_VERSION};
pub use leanvm_ipc_spawn::{
    encode_len_prefixed, read_len_prefixed, DEFAULT_IPC_WALL, SERVING_ENV, SPAWN_EXCHANGE_WIRED,
};
pub use leanvm_ipc_split::{decode_type1_leaves, encode_type1_leaves, split_ipc, split_ipc_at};
pub use signature::{verify, PublicKey, Signature};
pub use xmss::native::{
    OsRandom, RandomSource, SchemeParams, SeededRandom, XmssPublicKey, XmssSecretKey,
    XmssSignature, PROD as XMSS_PROD, TEST as XMSS_TEST,
};
pub use xmss::{
    assert_prod_invariants, key_gen, prod_fingerprint, sign, verify_bool, BASE, CONSTRUCTION,
    DIMENSION, LEANSIG_REV, LOG_LIFETIME, MESSAGE_BYTES, PROD_FINGERPRINT, PUBLIC_KEY_BYTES,
    SIGNATURE_BYTES, TARGET_SUM,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_invariants() {
        assert_prod_invariants();
        assert_aggregation_invariants();
        assert_eq!(PUBLIC_KEY_BYTES, 52);
        assert_eq!(SIGNATURE_BYTES, 2536);
        assert_eq!(MAX_PROOF_BYTES, 524_288);
        assert_eq!(LOG_INV_RATE, 2);
    }
}
