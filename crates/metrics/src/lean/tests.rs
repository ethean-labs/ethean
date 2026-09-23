use super::*;

struct DocMetric {
    name: String,
    kind: String,
    labels: Vec<String>,
    buckets: Vec<f64>,
}

/// Parse the metric tables of the vendored leanMetrics document.
fn doc_metrics() -> Vec<DocMetric> {
    let doc = include_str!("../../testdata/leanmetrics-69f9722.md");
    let mut header: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for line in doc.lines().filter(|l| l.starts_with('|')) {
        let cells: Vec<String> = line
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect();
        if cells.first().is_some_and(|c| c == "Name") {
            header = cells;
            continue;
        }
        let Some(name) = cells
            .first()
            .and_then(|c| c.strip_prefix('`'))
            .and_then(|c| c.strip_suffix('`'))
        else {
            continue;
        };
        let col = |title: &str| {
            header
                .iter()
                .position(|h| h == title)
                .map(|i| cells[i].clone())
                .unwrap_or_default()
        };
        let labels = col("Labels")
            .split("<br>")
            .flat_map(|part| match part.split_once('=') {
                Some((key, _)) => vec![key.trim().to_string()],
                None => part.split(',').map(|k| k.trim().to_string()).collect(),
            })
            .filter(|k| !k.is_empty())
            .collect();
        let buckets = col("Buckets")
            .split(',')
            .filter_map(|b| b.trim().parse::<f64>().ok())
            .collect();
        out.push(DocMetric {
            name: name.to_string(),
            kind: col("Type"),
            labels,
            buckets,
        });
    }
    out
}

#[test]
fn table_matches_the_leanmetrics_document() {
    let doc = doc_metrics();
    assert_eq!(
        doc.len(),
        LEAN_METRICS.len(),
        "every documented metric is implemented"
    );
    for d in &doc {
        let s = spec(&d.name).unwrap_or_else(|| panic!("{} missing", d.name));
        let kind = match s.kind {
            LeanKind::Counter => "Counter",
            LeanKind::Gauge => "Gauge",
            LeanKind::Histogram => "Histogram",
        };
        assert_eq!(kind, d.kind, "{} type", d.name);
        assert_eq!(
            s.labels,
            d.labels.iter().map(String::as_str).collect::<Vec<_>>(),
            "{} labels",
            d.name
        );
        assert_eq!(s.buckets, d.buckets.as_slice(), "{} buckets", d.name);
    }
}

#[test]
fn exposition_renders_counters_labels_and_histograms() {
    inc("lean_block_building_success_total", &[], 2.0);
    set("lean_node_sync_status", &["synced"], 1.0);
    observe("lean_attestation_validation_time_seconds", &[], 0.02);
    observe("lean_attestation_validation_time_seconds", &[], 5.0);
    let text = export_lean_text();
    assert!(text.contains("# TYPE lean_block_building_success_total counter"));
    assert!(text.contains("lean_node_sync_status{status=\"synced\"} 1"));
    assert!(text.contains("# TYPE lean_attestation_validation_time_seconds histogram"));
    assert!(text.contains("lean_attestation_validation_time_seconds_bucket{le=\"0.01\"} 0"));
    assert!(text.contains("lean_attestation_validation_time_seconds_bucket{le=\"0.025\"} 1"));
    assert!(text.contains("lean_attestation_validation_time_seconds_bucket{le=\"+Inf\"} 2"));
    assert!(text.contains("lean_attestation_validation_time_seconds_count 2"));
    assert!(
        text.contains("lean_head_slot 0"),
        "unlabeled metrics are always exposed"
    );
    assert!(
        !text.contains("lean_finalizations_total{"),
        "labeled series appear once recorded"
    );
}
