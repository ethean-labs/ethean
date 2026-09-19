//! Metrics schema version and closed label policy (OSD-009 leanMetrics pin still open).

/// Ethean metrics schema version embedded in build_info.
pub const METRICS_SCHEMA_VERSION: &str = "ethean-metrics-v1";

/// Metric name prefix (Ethean namespace; not peer-copied).
pub const METRIC_PREFIX: &str = "ethean_";

/// Forbidden high-cardinality label keys.
pub const FORBIDDEN_LABELS: &[&str] = &["root", "peer_id", "validator_index", "signature"];

/// Reject forbidden label names.
pub fn assert_label_allowed(name: &str) -> Result<(), crate::error::MetricsError> {
    if FORBIDDEN_LABELS.contains(&name) {
        return Err(crate::error::MetricsError::ForbiddenLabel(name.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_root_label() {
        assert!(assert_label_allowed("root").is_err());
        assert!(assert_label_allowed("slot_phase").is_ok());
    }
}
