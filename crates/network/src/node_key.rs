//! Persistent secp256k1 node key: the libp2p identity behind the QUIC PeerId.
//!
//! Hive and lean-quickstart hand every client a 32-byte secp256k1 secret
//! (64 hex chars, optional `0x`). Peers derive the bootnode multiaddr and ENR
//! from it, so the swarm must use exactly this key.

use crate::error::{NetworkError, Result};
use std::fs;
use std::path::Path;

/// 32-byte secp256k1 secret scalar.
#[derive(Clone, PartialEq, Eq)]
pub struct NodeKey {
    secret: [u8; 32],
}

impl std::fmt::Debug for NodeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeKey").field("secret", &"<redacted>").finish()
    }
}

impl NodeKey {
    /// Wrap raw secret bytes (all-zero is refused).
    pub fn from_bytes(secret: [u8; 32]) -> Result<Self> {
        if secret == [0u8; 32] {
            return Err(NetworkError::Handshake("node key must not be zero".into()));
        }
        #[cfg(feature = "libp2p-quic")]
        {
            let mut copy = secret;
            libp2p::identity::secp256k1::SecretKey::try_from_bytes(&mut copy)
                .map_err(|e| NetworkError::Handshake(format!("secp256k1 node key: {e}")))?;
        }
        Ok(Self { secret })
    }

    /// Parse 64 hex chars (optional `0x`, surrounding whitespace / newline tolerated).
    pub fn from_hex(text: &str) -> Result<Self> {
        let t = text.trim();
        let t = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
        if t.len() != 64 {
            return Err(NetworkError::Handshake(format!(
                "node key must be 64 hex chars, got {}",
                t.len()
            )));
        }
        let mut out = [0u8; 32];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = u8::from_str_radix(&t[i * 2..i * 2 + 2], 16)
                .map_err(|e| NetworkError::Handshake(format!("node key hex: {e}")))?;
        }
        Self::from_bytes(out)
    }

    /// Fresh random key.
    pub fn generate() -> Self {
        use rand::RngCore;
        loop {
            let mut secret = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut secret);
            if let Ok(key) = Self::from_bytes(secret) {
                return key;
            }
        }
    }

    /// Lowercase hex without prefix (file format).
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for b in &self.secret {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }

    /// Raw secret bytes.
    pub fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret
    }

    /// Read a hex key file.
    pub fn load_file(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .map_err(|e| NetworkError::Handshake(format!("read {}: {e}", path.display())))?;
        Self::from_hex(&text)
            .map_err(|e| NetworkError::Handshake(format!("{}: {e}", path.display())))
    }

    /// Write the key as hex (+ newline), owner-only permissions on Unix.
    pub fn save_file(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| {
                NetworkError::Handshake(format!("create {}: {e}", dir.display()))
            })?;
        }
        let mut text = self.to_hex();
        text.push('\n');
        fs::write(path, text)
            .map_err(|e| NetworkError::Handshake(format!("write {}: {e}", path.display())))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    /// Load `path` when present, otherwise generate and persist. Returns `(key, created)`.
    pub fn load_or_create(path: &Path) -> Result<(Self, bool)> {
        if path.is_file() {
            return Ok((Self::load_file(path)?, false));
        }
        let key = Self::generate();
        key.save_file(path)?;
        Ok((key, true))
    }

    /// libp2p keypair (secp256k1) for the swarm identity.
    #[cfg(feature = "libp2p-quic")]
    pub fn libp2p_keypair(&self) -> Result<libp2p::identity::Keypair> {
        let mut copy = self.secret;
        let sk = libp2p::identity::secp256k1::SecretKey::try_from_bytes(&mut copy)
            .map_err(|e| NetworkError::Handshake(format!("secp256k1 node key: {e}")))?;
        Ok(libp2p::identity::secp256k1::Keypair::from(sk).into())
    }

    /// PeerId derived from this key.
    #[cfg(feature = "libp2p-quic")]
    pub fn peer_id(&self) -> Result<libp2p::PeerId> {
        Ok(self.libp2p_keypair()?.public().to_peer_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAM_0: &str = "af27950128b49cda7e7bc9fcb7b0270f7a3945aa7543326f3bfdbd57d2a97a32";

    #[test]
    fn parses_hex_with_prefix_and_newline() {
        let a = NodeKey::from_hex(REAM_0).unwrap();
        let b = NodeKey::from_hex(&format!("0x{REAM_0}\n")).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.to_hex(), REAM_0);
        assert!(NodeKey::from_hex("abcd").is_err());
        assert!(NodeKey::from_hex(&"00".repeat(32)).is_err());
    }

    #[test]
    fn load_or_create_round_trip() {
        let dir = std::env::temp_dir().join(format!("ethean-node-key-{}", std::process::id()));
        let path = dir.join("node.key");
        let _ = fs::remove_dir_all(&dir);
        let (k1, created) = NodeKey::load_or_create(&path).unwrap();
        assert!(created);
        let (k2, created) = NodeKey::load_or_create(&path).unwrap();
        assert!(!created);
        assert_eq!(k1, k2);
        assert_eq!(fs::read_to_string(&path).unwrap().trim().len(), 64);
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(feature = "libp2p-quic")]
    #[test]
    fn peer_id_matches_lean_quickstart_vector() {
        // lean-quickstart local-devnet validator-config.yaml, node ream_0.
        let key = NodeKey::from_hex(REAM_0).unwrap();
        assert_eq!(
            key.peer_id().unwrap().to_string(),
            "16Uiu2HAmPQhkD6Zg5Co2ee8ShshkiY4tDePKFARPpCS2oKSLj1E1"
        );
    }
}
