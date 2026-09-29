//! XMSS wire signature types (PROD sizes).

mod public_key;
mod verify;
mod wire;

pub use public_key::PublicKey;
pub use verify::verify;
pub use wire::Signature;
