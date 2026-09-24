//! Operator aggregation / leanVM pin checks (B4).

use ethean_crypto::{LEANSIG_REV, LEANVM_REV, LOG_INV_RATE};
use std::fs;
use std::path::{Path, PathBuf};

/// Keys we compare against the local production fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AggPin {
    /// Operator `LOG_INV_RATE` when present.
    pub log_inv_rate: Option<u32>,
    /// Operator leanVM git rev (40 hex) when present.
    pub leanvm_rev: Option<String>,
    /// Operator leanSig git rev (40 hex) when present.
    pub leansig_rev: Option<String>,
}

/// Mismatch between operator pin and this build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggPinMismatch {
    /// Field name (`LOG_INV_RATE`, `LEANVM_REV`, `LEANSIG_REV`).
    pub field: &'static str,
    /// Operator value.
    pub operator: String,
    /// Local build value.
    pub local: String,
}

impl AggPin {
    /// Parse `KEY=value` lines (comments `#` and blanks ignored).
    pub fn parse(text: &str) -> Self {
        let mut pin = Self::default();
        for raw in text.lines() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let key = k.trim().to_ascii_uppercase();
            let val = v.trim();
            match key.as_str() {
                "LOG_INV_RATE" => {
                    if let Ok(n) = val.parse::<u32>() {
                        pin.log_inv_rate = Some(n);
                    }
                }
                "LEANVM_REV" => {
                    if !val.is_empty() {
                        pin.leanvm_rev = Some(val.to_string());
                    }
                }
                "LEANSIG_REV" => {
                    if !val.is_empty() {
                        pin.leansig_rev = Some(val.to_string());
                    }
                }
                _ => {}
            }
        }
        pin
    }

    /// Load from path when the file exists.
    pub fn load_file(path: &Path) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        Some(Self::parse(&text))
    }

    /// Diff against this build's production constants.
    pub fn mismatches(&self) -> Vec<AggPinMismatch> {
        let mut out = Vec::new();
        if let Some(rate) = self.log_inv_rate {
            if rate != LOG_INV_RATE {
                out.push(AggPinMismatch {
                    field: "LOG_INV_RATE",
                    operator: rate.to_string(),
                    local: LOG_INV_RATE.to_string(),
                });
            }
        }
        if let Some(ref rev) = self.leanvm_rev {
            if rev != LEANVM_REV {
                out.push(AggPinMismatch {
                    field: "LEANVM_REV",
                    operator: rev.clone(),
                    local: LEANVM_REV.to_string(),
                });
            }
        }
        if let Some(ref rev) = self.leansig_rev {
            if rev != LEANSIG_REV {
                out.push(AggPinMismatch {
                    field: "LEANSIG_REV",
                    operator: rev.clone(),
                    local: LEANSIG_REV.to_string(),
                });
            }
        }
        out
    }
}

/// Default path under `config/networks/{stem}.aggpin`.
pub fn aggpin_path_for_stem(stem: &str) -> PathBuf {
    PathBuf::from(format!("config/networks/{stem}.aggpin"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_match_local_pins() {
        let text = format!(
            "LOG_INV_RATE={LOG_INV_RATE}\nLEANVM_REV={LEANVM_REV}\nLEANSIG_REV={LEANSIG_REV}\n"
        );
        let pin = AggPin::parse(&text);
        assert!(pin.mismatches().is_empty());
    }

    #[test]
    fn detects_rate_mismatch() {
        let pin = AggPin::parse("LOG_INV_RATE=99\n");
        let m = pin.mismatches();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].field, "LOG_INV_RATE");
    }
}
