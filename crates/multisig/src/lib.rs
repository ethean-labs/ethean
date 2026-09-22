//! leanMultisig aggregate proofs for Ethean.
//!
//! - [`verify`]: in-process Type-1 / Type-2 verification with bounded decoding
//!   and panic isolation, exposed as [`LeanMultisigVerifier`].
//! - [`prove`]: Type-1 aggregation, Type-2 merge and split. These run only in
//!   the `ethean-prover` child process (see [`client`] and [`protocol`]).
//!
//! Pinned to leanVM `e2592df4` (pq-devnet-4), the same revision ream,
//! ethlambda and zeam link, so proofs are byte-compatible across clients.

pub mod client;
mod error;
mod frame;
mod isolate;
mod keys;
pub mod limits;
pub mod protocol;
pub mod prove;
pub mod verify;

pub use client::{ProverClient, ProverConfig};
pub use error::{MultisigError, Result};
pub use prove::KeyedProof;
pub use verify::{init_verifier, verify_multi, verify_single, LeanMultisigVerifier};

/// leanVM revision these proofs are produced and verified with.
pub const LEANVM_REV: &str = "e2592df4e30fdddbbf8ae26a333116c68cec7026";
