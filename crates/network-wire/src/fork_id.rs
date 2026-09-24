//! Gossip fork-digest segment for Lean topics.

use crate::error::{Result, WireError};

/// lstar `GOSSIP_DIGEST` (leanSpec `LstarSpec.GOSSIP_DIGEST`).
pub const LSTAR_GOSSIP_DIGEST: &str = "12345678";

/// Default gossip/req fork segment for a named fork.
///
/// lstar uses the hardcoded digest `"12345678"`. Other names keep an 8-hex
/// placeholder derived from the name bytes (operator `--fork-digest` still
/// overrides). Topic strings never include a `0x` prefix (leanSpec).
pub fn fork_segment_from_name(fork_name: &str) -> Result<String> {
    if fork_name.is_empty() {
        return Err(WireError::InvalidTopic("empty fork name".into()));
    }
    if fork_name.eq_ignore_ascii_case("lstar") {
        return Ok(LSTAR_GOSSIP_DIGEST.to_string());
    }
    // Non-lstar forks: keep a stable 8-hex fallback until the fork pins GOSSIP_DIGEST.
    let mut out = [0u8; 4];
    let bytes = fork_name.as_bytes();
    for (i, b) in bytes.iter().take(4).enumerate() {
        out[i] = *b;
    }
    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}",
        out[0], out[1], out[2], out[3]
    ))
}

/// Hex encoding of the default segment for `fork_name`.
pub fn fork_segment_hex(fork_name: &str) -> Result<String> {
    fork_segment_from_name(fork_name)
}

/// Four identifier bytes for the default segment of `fork_name`.
pub fn fork_identifier_bytes(fork_name: &str) -> Result<[u8; 4]> {
    let hex = fork_segment_from_name(fork_name)?;
    parse_digest_hex(&hex)
}

/// Parse an 8-hex digest, accepting an optional `0x` / `0X` prefix.
pub fn parse_digest_hex(raw: &str) -> Result<[u8; 4]> {
    let hex = normalize_digest_hex(raw)?;
    let mut out = [0u8; 4];
    for i in 0..4 {
        out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| WireError::InvalidTopic(format!("fork digest hex '{raw}'")))?;
    }
    Ok(out)
}

/// Strip an optional `0x` prefix and lowercase an 8-hex digest.
pub fn normalize_digest_hex(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed)
        .to_ascii_lowercase();
    if hex.len() != 8 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(WireError::InvalidTopic(format!(
            "fork digest override must be 8 hex chars, got '{raw}'"
        )));
    }
    Ok(hex)
}

/// Resolve the gossip fork segment: operator override hex wins over the fork default.
///
/// `override_hex` may include a `0x` prefix (accepted on input, stripped on output).
/// Topic strings always use the unprefixed 8-hex form that leanSpec emits.
pub fn fork_segment_resolve(fork_name: &str, override_hex: Option<&str>) -> Result<String> {
    if let Some(raw) = override_hex {
        if !raw.trim().is_empty() {
            return normalize_digest_hex(raw);
        }
    }
    fork_segment_from_name(fork_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lstar_default_is_gossip_digest() {
        assert_eq!(fork_segment_from_name("lstar").unwrap(), LSTAR_GOSSIP_DIGEST);
        assert_eq!(fork_segment_hex("lstar").unwrap(), "12345678");
        assert_eq!(
            fork_identifier_bytes("lstar").unwrap(),
            [0x12, 0x34, 0x56, 0x78]
        );
    }

    #[test]
    fn rejects_empty() {
        assert!(fork_identifier_bytes("").is_err());
    }

    #[test]
    fn override_wins_and_strips_0x() {
        let over = fork_segment_resolve("lstar", Some("0xAABBCCDD")).unwrap();
        assert_eq!(over, "aabbccdd");
        assert_ne!(over, LSTAR_GOSSIP_DIGEST);
    }

    #[test]
    fn accepts_lstar_digest_override() {
        assert_eq!(
            fork_segment_resolve("lstar", Some("12345678")).unwrap(),
            "12345678"
        );
        assert_eq!(
            fork_segment_resolve("lstar", Some("0x12345678")).unwrap(),
            "12345678"
        );
    }
}
