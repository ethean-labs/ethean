//! Ethean Lean Consensus crypto — XMSS wire types, aggregation, backends.
//!
//! Production XMSS / leanVM paths fail closed unless optional backends compile.
//! Default test helpers verify with real bindings (never always-true).

#![forbid(unsafe_code)]

pub mod aggregation;
pub mod backend;
#[cfg(feature = "leansig-backend")]
mod backend_leansig;
#[cfg(feature = "leanvm-backend")]
mod backend_leanvm;
pub mod domain;
pub mod error;
pub mod ffi_status;
pub mod hash;
pub mod leanvm_ipc;
mod leanvm_ipc_frame;
mod leanvm_ipc_spawn;
pub mod signature;
pub mod xmss;

pub use aggregation::{
    aggregation_fingerprint, assert_aggregation_invariants, attestation_leaves_from_type2,
    merge_type1, merge_type1_statements, prove_type1, prove_type2, split_type2_to_type1,
    verify_statement_shape, verify_type1, verify_type2, AggregateStatement, ParticipantSet,
    ProofKind, Type1Leaf, Type2ComponentRef, LEANVM_REV, LOG_INV_RATE, MAX_PROOF_BYTES,
    PROD_AGGREGATION_FINGERPRINT,
};
pub use backend::{CryptoBackend, ProductionBackend, SecretKeyMaterial};
#[cfg(any(test, feature = "test-hmac"))]
pub use backend::TestHmacBackend;
pub use error::{CryptoError, Result};
pub use ffi_status::{BackendGap, FfiStatus, LeanSigGate, LeanVmGate};
pub use leanvm_ipc::{
    try_roundtrip_prove, LeanVmIpcStatus, PROVER_ENV as LEANVM_PROVER_ENV,
};
pub use leanvm_ipc_frame::{
    IpcFrame, IpcOp, FRAME_CODEC_READY, FRAME_MAGIC, FRAME_VERSION,
};
pub use leanvm_ipc_spawn::{
    encode_len_prefixed, read_len_prefixed, SPAWN_EXCHANGE_WIRED, DEFAULT_IPC_WALL,
};
pub use hash::{domain_digest, signature_hash, signing_root_digest, Digest32};
pub use signature::{verify, PublicKey, Signature};
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
