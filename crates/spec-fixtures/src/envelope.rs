//! Serde shapes for leanSpec filled JSON fixtures (partial).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One filled fixture file: map of pytest id → case body.
#[derive(Debug, Clone, Deserialize)]
pub struct FixtureFile {
    /// Cases keyed by the leanSpec pytest node id.
    #[serde(flatten)]
    pub cases: std::collections::BTreeMap<String, FixtureCase>,
}

/// One consensus vector case (fields beyond steps remain opaque for now).
#[derive(Debug, Clone, Deserialize)]
pub struct FixtureCase {
    /// Fork / network name (`Lstar`, …).
    #[serde(default)]
    pub network: Option<String>,
    /// `prod` / `test` leanEnv.
    #[serde(rename = "leanEnv", default)]
    pub lean_env: Option<String>,
    /// Ordered steps (tick / block / checks).
    #[serde(default)]
    pub steps: Vec<FixtureStep>,
    /// Remainder kept for later runners (anchorState, …).
    #[serde(flatten)]
    pub rest: std::collections::BTreeMap<String, Value>,
}

/// One step inside a fixture case.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FixtureStep {
    /// Whether the step is expected to succeed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
    /// leanSpec `SpecRejectionError` name when `valid` is false.
    #[serde(rename = "rejectionReason", default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    /// Opaque checks / block payloads until typed runners land.
    #[serde(flatten)]
    pub rest: std::collections::BTreeMap<String, Value>,
}

impl FixtureFile {
    /// Parse JSON bytes from a filled leanSpec fixture file.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// First rejected step reason across all cases (smoke helper).
    pub fn first_rejection_reason(&self) -> Option<&str> {
        for case in self.cases.values() {
            for step in &case.steps {
                if step.valid == Some(false) {
                    if let Some(r) = step.rejection_reason.as_deref() {
                        return Some(r);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_committed_sample() {
        let raw = include_bytes!(
            "../../../spec/fixtures/samples/fork_choice/block_beyond_future_horizon_rejected.json"
        );
        let file = FixtureFile::from_slice(raw).expect("sample json");
        assert!(!file.cases.is_empty());
        assert_eq!(
            file.first_rejection_reason(),
            Some("BLOCK_TOO_FAR_IN_FUTURE")
        );
    }
}
