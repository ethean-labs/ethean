//! Prometheus text exposition for the leanMetrics series.

use super::spec::{spec, LeanKind};
use super::store::{with_store, Series};

fn label_block(keys: &[&str], values: &[String], extra: Option<(&str, String)>) -> String {
    let mut pairs: Vec<String> = keys
        .iter()
        .zip(values)
        .map(|(k, v)| format!("{k}=\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();
    if let Some((k, v)) = extra {
        pairs.push(format!("{k}=\"{v}\""));
    }
    if pairs.is_empty() {
        String::new()
    } else {
        format!("{{{}}}", pairs.join(","))
    }
}

fn number(v: f64) -> String {
    if v.is_infinite() {
        "+Inf".into()
    } else {
        format!("{v}")
    }
}

/// Render every leanMetrics series in Prometheus text format.
pub fn export_lean_text() -> String {
    with_store(|store| {
        let mut out = String::new();
        for (name, by_labels) in &store.series {
            let spec = spec(name).expect("series come from the spec table");
            let kind = match spec.kind {
                LeanKind::Counter => "counter",
                LeanKind::Gauge => "gauge",
                LeanKind::Histogram => "histogram",
            };
            out.push_str(&format!(
                "# HELP {name} {}\n# TYPE {name} {kind}\n",
                spec.help
            ));
            for (values, series) in by_labels {
                match series {
                    Series::Scalar(v) => {
                        let labels = label_block(spec.labels, values, None);
                        out.push_str(&format!("{name}{labels} {}\n", number(*v)));
                    }
                    Series::Histogram { counts, sum, count } => {
                        for (upper, c) in spec.buckets.iter().zip(counts) {
                            let labels =
                                label_block(spec.labels, values, Some(("le", number(*upper))));
                            out.push_str(&format!("{name}_bucket{labels} {c}\n"));
                        }
                        let inf = label_block(spec.labels, values, Some(("le", "+Inf".into())));
                        let plain = label_block(spec.labels, values, None);
                        out.push_str(&format!("{name}_bucket{inf} {count}\n"));
                        out.push_str(&format!("{name}_sum{plain} {}\n", number(*sum)));
                        out.push_str(&format!("{name}_count{plain} {count}\n"));
                    }
                }
            }
        }
        out
    })
}
