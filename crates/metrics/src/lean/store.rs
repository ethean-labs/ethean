//! Process-wide store for the leanMetrics series.
//!
//! Recording happens from many threads (chain owner, proof service, scrape),
//! so the store is a global behind a mutex; every call is a few map lookups.
//! Unknown names or wrong label arity are programming errors: they panic in
//! debug builds and are dropped in release builds.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use super::spec::{spec, LeanKind, LeanSpec, LEAN_METRICS};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Series {
    Scalar(f64),
    Histogram {
        counts: Vec<u64>,
        sum: f64,
        count: u64,
    },
}

impl Series {
    fn empty(spec: &LeanSpec) -> Self {
        match spec.kind {
            LeanKind::Histogram => Series::Histogram {
                counts: vec![0; spec.buckets.len()],
                sum: 0.0,
                count: 0,
            },
            _ => Series::Scalar(0.0),
        }
    }
}

/// All series, keyed by metric name then label values.
#[derive(Debug, Default)]
pub(crate) struct Store {
    pub(crate) series: BTreeMap<&'static str, BTreeMap<Vec<String>, Series>>,
}

impl Store {
    fn new() -> Self {
        let mut store = Self::default();
        for spec in LEAN_METRICS
            .iter()
            .chain(super::spec_extra::LEAN_METRICS_EXTRA.iter())
        {
            let mut by_labels = BTreeMap::new();
            if spec.labels.is_empty() {
                by_labels.insert(Vec::new(), Series::empty(spec));
            }
            store.series.insert(spec.name, by_labels);
        }
        store
    }

    fn entry(&mut self, name: &str, labels: &[&str]) -> Option<(&'static LeanSpec, &mut Series)> {
        let Some(spec) = spec(name) else {
            debug_assert!(false, "unknown lean metric {name}");
            return None;
        };
        if labels.len() != spec.labels.len() {
            debug_assert!(false, "{name} expects labels {:?}", spec.labels);
            return None;
        }
        let key: Vec<String> = labels.iter().map(|l| l.to_string()).collect();
        let series = self
            .series
            .get_mut(spec.name)?
            .entry(key)
            .or_insert_with(|| Series::empty(spec));
        Some((spec, series))
    }
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(Store::new()))
}

pub(crate) fn with_store<R>(f: impl FnOnce(&mut Store) -> R) -> R {
    let mut guard = store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

/// Add `delta` to a counter (or gauge).
pub fn inc(name: &str, labels: &[&str], delta: f64) {
    with_store(|s| {
        if let Some((_, Series::Scalar(v))) = s.entry(name, labels) {
            *v += delta;
        }
    });
}

/// Set a gauge.
pub fn set(name: &str, labels: &[&str], value: f64) {
    with_store(|s| {
        if let Some((_, Series::Scalar(v))) = s.entry(name, labels) {
            *v = value;
        }
    });
}

/// Record one histogram observation.
pub fn observe(name: &str, labels: &[&str], value: f64) {
    with_store(|s| {
        if let Some((spec, Series::Histogram { counts, sum, count })) = s.entry(name, labels) {
            for (bucket, upper) in counts.iter_mut().zip(spec.buckets) {
                if value <= *upper {
                    *bucket += 1;
                }
            }
            *sum += value;
            *count += 1;
        }
    });
}

/// Current value of a scalar series (tests and dashboards helpers).
pub fn value(name: &str, labels: &[&str]) -> Option<f64> {
    with_store(|s| match s.entry(name, labels) {
        Some((_, Series::Scalar(v))) => Some(*v),
        _ => None,
    })
}

/// Observation count of a histogram series.
pub fn observations(name: &str, labels: &[&str]) -> Option<u64> {
    with_store(|s| match s.entry(name, labels) {
        Some((_, Series::Histogram { count, .. })) => Some(*count),
        _ => None,
    })
}
