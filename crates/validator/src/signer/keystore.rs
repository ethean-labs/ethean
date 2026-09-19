//! Key records — secrets never Debug-printed.

use crate::signer::duty::{KeyId, SigningRole};
use ethean_crypto::{PublicKey, SecretKeyMaterial};

/// Imported key metadata + secret handle.
#[derive(Clone)]
pub struct KeyRecord {
    /// Local key id.
    pub key_id: KeyId,
    /// Role this key may serve.
    pub role: SigningRole,
    /// Public key (52 bytes wire).
    pub public_key: PublicKey,
    /// Secret material (redacted in Debug).
    pub secret: SecretKeyMaterial,
    /// First signable slot.
    pub activation_slot: u32,
    /// Number of active slots.
    pub num_active_slots: u32,
    /// Monotonic journal generation for this key.
    pub journal_generation: u64,
}

impl std::fmt::Debug for KeyRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyRecord")
            .field("key_id", &self.key_id)
            .field("role", &self.role)
            .field("public_key", &self.public_key)
            .field("secret", &"<redacted>")
            .field("activation_slot", &self.activation_slot)
            .field("num_active_slots", &self.num_active_slots)
            .field("journal_generation", &self.journal_generation)
            .finish()
    }
}
