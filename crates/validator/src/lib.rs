//! Ethean Lean Consensus validator — durable XMSS signer safety.

#![forbid(unsafe_code)]

pub mod error;
pub mod signer;

pub use error::{Result, SignerError};
pub use signer::{
    InMemorySignerStore, KeyId, KeyRecord, ReservationStatus, Signer, SignerStore, SigningDuty,
    SigningRole, SigningRoot,
};
