//! Domain-separated digests (SHA-256 helpers; not production Poseidon).

mod digest;

pub use digest::{domain_digest, signature_hash, signing_root_digest, Digest32};
