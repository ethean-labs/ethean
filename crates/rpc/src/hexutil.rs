//! Lowercase `0x`-prefixed root encoding used by the hive `/lean/v0` contract.

use ethean_primitives::Hash32;

/// Format a 32-byte root as `0x` + 64 lowercase hex chars.
pub fn hex_root_0x(root: &Hash32) -> String {
    let mut out = String::with_capacity(66);
    out.push_str("0x");
    for b in root {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Parse `0x`-prefixed or bare 64-char hex into a root.
pub fn parse_hex_root(s: &str) -> Result<Hash32, String> {
    let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    if hex.len() != 64 {
        return Err(format!("expected 32-byte hex, got len {}", hex.len()));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| "invalid hex digit".to_string())?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::HASH32_ZERO;

    #[test]
    fn encodes_zero_with_prefix() {
        let s = hex_root_0x(&HASH32_ZERO);
        assert!(s.starts_with("0x"));
        assert_eq!(s.len(), 66);
        assert_eq!(s, "0x0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(parse_hex_root(&s).unwrap(), HASH32_ZERO);
    }
}
