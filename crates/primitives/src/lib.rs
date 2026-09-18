//! Ethean Lean Consensus primitives.
//!
//! Small, stable value types with no protocol orchestration, I/O, or chain config.

#![forbid(unsafe_code)]

mod bytes;
mod epoch;
mod error;
mod hash;
mod slot;
mod validator;

pub use bytes::{Bytes52, XMSS_PUBLIC_KEY_BYTES};
pub use epoch::Epoch;
pub use error::PrimitiveError;
pub use hash::{is_zero, Hash32, HASH32_ZERO};
pub use slot::Slot;
pub use validator::ValidatorIndex;
