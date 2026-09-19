//! Typed signing duties and roles (role confusion unrepresentable).

/// Stable key identifier (16 bytes).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyId([u8; 16]);

impl KeyId {
    /// Construct from exact bytes.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Borrow bytes.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl std::fmt::Debug for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyId(..)")
    }
}

/// Signing role; attestation and proposal keys stay separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SigningRole {
    /// Attestation / vote signatures.
    Attestation,
    /// Block proposal signatures.
    Proposal,
}

/// Canonical 32-byte signing root.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SigningRoot([u8; 32]);

impl SigningRoot {
    /// Construct from bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrow as message bytes for the crypto backend.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for SigningRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SigningRoot(..)")
    }
}

/// One signing request bound to role, slot, key, and root.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SigningDuty {
    /// Key that must sign.
    pub key_id: KeyId,
    /// Role expected of that key.
    pub role: SigningRole,
    /// Slot / epoch leaf index.
    pub slot: u32,
    /// Message root to sign.
    pub root: SigningRoot,
}

/// Reservation lifecycle for a duty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservationStatus {
    /// Fresh reservation written (needs flush before sign).
    Reserved,
    /// Reservation exists but flush has not completed.
    PendingFlush,
    /// Same duty already completed (idempotent path).
    AlreadyCompleted,
    /// Same role/slot with a different root.
    Conflict,
}
