//! Prometheus text exposition for the in-process registry.

use crate::registry::{MetricKind, Registry};

/// Render Prometheus text format (no timestamps).
pub fn export_prometheus_text(reg: &Registry) -> String {
    let mut out = String::new();
    let mut names: Vec<_> = reg.samples().map(|s| s.name.clone()).collect();
    names.sort();
    for name in names {
        let sample = reg.samples().find(|s| s.name == name).unwrap();
        out.push_str(&format!("# HELP {} {}\n", sample.name, sample.help));
        let kind = match sample.kind {
            MetricKind::Counter => "counter",
            MetricKind::Gauge => "gauge",
        };
        out.push_str(&format!("# TYPE {} {}\n", sample.name, kind));
        if sample.labels.is_empty() {
            out.push_str(&format!("{} {}\n", sample.name, sample.value));
        } else {
            let labels: Vec<String> = sample
                .labels
                .iter()
                .map(|(k, v)| format!("{k}=\"{v}\""))
                .collect();
            out.push_str(&format!(
                "{}{{{}}} {}\n",
                sample.name,
                labels.join(","),
                sample.value
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{MetricKind, Registry};

    #[test]
    fn includes_build_info() {
        let r = Registry::with_build_info("0.1.0").unwrap();
        let text = export_prometheus_text(&r);
        assert!(text.contains("ethean_build_info"));
        assert!(text.contains("schema=\"ethean-metrics-v1\""));
    }

    #[test]
    fn exports_gauge() {
        let mut r = Registry::with_build_info("0.1.0").unwrap();
        r.register("ready", MetricKind::Gauge, "Ready", 0.0, vec![])
            .unwrap();
        assert!(export_prometheus_text(&r).contains("ethean_ready 0"));
    }
}
