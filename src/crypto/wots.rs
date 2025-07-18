//! WOTS+ (Winternitz One-Time Signature Plus) implementation
//!
//! Post-quantum signature scheme for Beam Chain.
//! Modular design: key generation, signing, verification in separate modules.

pub mod keygen;
pub mod sign;
pub mod verify;
pub mod params;

pub use keygen::WotsKeyPair;
pub use sign::WotsSigner;
pub use verify::WotsVerifier;
pub use params::WotsParams;

/// WOTS signature type - variable length based on parameters
pub type WotsSignature = Vec<u8>;

/// WOTS public key type
pub type WotsPublicKey = Vec<u8>;

/// WOTS private key type  
pub type WotsPrivateKey = Vec<u8>;

/// Main WOTS interface
pub struct Wots {
    pub params: WotsParams,
}

impl Wots {
    /// Create new WOTS instance with default parameters
    pub fn new() -> Self {
        Self {
            params: WotsParams::default(),
        }
    }

    /// Create WOTS with custom parameters
    pub fn with_params(params: WotsParams) -> Self {
        Self { params }
    }

    /// Generate new keypair
    pub fn generate_keypair(&self) -> WotsKeyPair {
        keygen::generate_keypair(&self.params)
    }

    /// Sign message
    pub fn sign(&self, message: &[u8], private_key: &WotsPrivateKey) -> WotsSignature {
        sign::sign_message(message, private_key, &self.params)
    }

    /// Verify signature
    pub fn verify(&self, message: &[u8], signature: &WotsSignature, public_key: &WotsPublicKey) -> bool {
        verify::verify_signature(message, signature, public_key, &self.params)
    }
}

impl Default for Wots {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wots_basic_flow() {
        let wots = Wots::new();
        let keypair = wots.generate_keypair();
        
        let message = b"Hello Beam Chain!";
        let signature = wots.sign(message, &keypair.private_key);
        let is_valid = wots.verify(message, &signature, &keypair.public_key);
        
        assert!(is_valid);
    }

    #[test]
    fn test_wots_invalid_signature() {
        let wots = Wots::new();
        let keypair = wots.generate_keypair();
        
        let message = b"Hello Beam Chain!";
        let wrong_message = b"Wrong message!";
        let signature = wots.sign(message, &keypair.private_key);
        let is_valid = wots.verify(wrong_message, &signature, &keypair.public_key);
        
        assert!(!is_valid);
    }
}
