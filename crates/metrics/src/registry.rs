//! In-process metric registry with closed names.

use crate::error::{MetricsError, Result};
use crate::schema::{assert_label_allowed, METRICS_SCHEMA_VERSION, METRIC_PREFIX};
use std::collections::HashMap;

/// Metric kinds we expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Counter,
    Gauge,
}

/// Registered metric metadata + value.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricSample {
    pub name: String,
    pub kind: MetricKind,
    pub help: String,
    pub value: f64,
    /// Closed label pairs (name already validated).
    pub labels: Vec<(String, String)>,
}

/// Registry of ethean_ metrics.
#[derive(Debug, Default)]
pub struct Registry {
    metrics: HashMap<String, MetricSample>,
}

impl Registry {
    /// Create with build_info gauge.
    pub fn with_build_info(version: &str) -> Result<Self> {
        let mut reg = Self::default();
        reg.register(
            "build_info",
            MetricKind::Gauge,
            "Build and metrics schema version",
            1.0,
            vec![
                ("version".into(), version.into()),
                ("schema".into(), METRICS_SCHEMA_VERSION.into()),
            ],
        )?;
        Ok(reg)
    }

    /// Register a new metric (names get ethean_ prefix).
    pub fn register(
        &mut self,
        short_name: &str,
        kind: MetricKind,
        help: &str,
        value: f64,
        labels: Vec<(String, String)>,
    ) -> Result<()> {
        for (k, _) in &labels {
            assert_label_allowed(k)?;
        }
        let name = format!("{METRIC_PREFIX}{short_name}");
        if self.metrics.contains_key(&name) {
            return Err(MetricsError::DuplicateMetric(name));
        }
        self.metrics.insert(
            name.clone(),
            MetricSample {
                name,
                kind,
                help: help.to_string(),
                value,
                labels,
            },
        );
        Ok(())
    }

    /// Set a gauge/counter value.
    pub fn set(&mut self, short_name: &str, value: f64) -> Result<()> {
        let name = format!("{METRIC_PREFIX}{short_name}");
        let m = self
            .metrics
            .get_mut(&name)
            .ok_or_else(|| MetricsError::UnknownMetric(name))?;
        m.value = value;
        Ok(())
    }

    /// Increment a counter.
    pub fn inc(&mut self, short_name: &str, delta: f64) -> Result<()> {
        let name = format!("{METRIC_PREFIX}{short_name}");
        let m = self
            .metrics
            .get_mut(&name)
            .ok_or_else(|| MetricsError::UnknownMetric(name))?;
        m.value += delta;
        Ok(())
    }

    /// Iterate samples.
    pub fn samples(&self) -> impl Iterator<Item = &MetricSample> {
        self.metrics.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_and_forbidden_label() {
        let mut r = Registry::with_build_info("0.1.0").unwrap();
        assert!(r
            .register(
                "x",
                MetricKind::Gauge,
                "h",
                0.0,
                vec![("root".into(), "abc".into())]
            )
            .is_err());
        r.register("head_slot", MetricKind::Gauge, "Head slot", 1.0, vec![])
            .unwrap();
        assert!(r
            .register("head_slot", MetricKind::Gauge, "Head slot", 2.0, vec![])
            .is_err());
    }
}
