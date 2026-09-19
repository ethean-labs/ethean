//! Interim fork identifier bytes until Phase 00 OSD pins the digest.

use sha2::{Digest, Sha256};

use crate::error::{Result, WireError};
use crate::topics::FORBIDDEN_DUMMY_FORK;

/// First four SHA-256 bytes of `fork_name` (interim; not a leanSpec wire pin).
pub fn fork_identifier_bytes(fork_name: &str) -> Result<[u8; 4]> {
    if fork_name.is_empty() {
        return Err(WireError::InvalidTopic("empty fork name".into()));
    }
    let digest = Sha256::digest(fork_name.as_bytes());
    let mut out = [0u8; 4];
    out.copy_from_slice(&digest[..4]);
    let hex = format!(
        "{:02x}{:02x}{:02x}{:02x}",
        out[0], out[1], out[2], out[3]
    );
    if hex == FORBIDDEN_DUMMY_FORK {
        return Err(WireError::InvalidTopic(
            "fork identifier collided with forbidden dummy".into(),
        ));
    }
    Ok(out)
}

/// Hex encoding of [`fork_identifier_bytes`] for topic path segments.
pub fn fork_segment_hex(fork_name: &str) -> Result<String> {
    let bytes = fork_identifier_bytes(fork_name)?;
    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    ))
}

/// Resolve the gossip/req fork segment: operator override hex wins over interim name hash.
///
/// `override_hex` must be exactly 8 hex chars (4 bytes), optional `0x` prefix; not the dummy id.
pub fn fork_segment_resolve(fork_name: &str, override_hex: Option<&str>) -> Result<String> {
    if let Some(raw) = override_hex {
        let hex = raw.trim().trim_start_matches("0x").to_ascii_lowercase();
        if hex.len() != 8 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(WireError::InvalidTopic(format!(
                "fork digest override must be 8 hex chars, got '{raw}'"
            )));
        }
        if hex == FORBIDDEN_DUMMY_FORK {
            return Err(WireError::InvalidTopic(
                "fork digest override is the forbidden dummy".into(),
            ));
        }
        return Ok(hex);
    }
    fork_segment_hex(fork_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_bytes_and_hex() {
        let a = fork_identifier_bytes("lstar").unwrap();
        let b = fork_identifier_bytes("lstar").unwrap();
        assert_eq!(a, b);
        assert_eq!(fork_segment_hex("lstar").unwrap().len(), 8);
    }

    #[test]
    fn rejects_empty() {
        assert!(fork_identifier_bytes("").is_err());
    }

    #[test]
    fn override_wins() {
        let interim = fork_segment_hex("lstar").unwrap();
        let over = fork_segment_resolve("lstar", Some("0xAABBCCDD")).unwrap();
        assert_eq!(over, "aabbccdd");
        assert_ne!(over, interim);
    }

    #[test]
    fn rejects_dummy_override() {
        assert!(fork_segment_resolve("lstar", Some("12345678")).is_err());
    }
}
