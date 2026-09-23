//! leanMetrics standard metrics (`lean_*`), exported next to the `ethean_*`
//! families so Ethean appears on the shared lean client dashboards.

mod export;
mod spec;
mod store;

pub use export::export_lean_text;
pub use spec::{spec, LeanKind, LeanSpec, LEAN_METRICS};
pub use store::{inc, observations, observe, set, value};

/// Record wall time since `start` into a histogram.
pub fn observe_since(name: &str, labels: &[&str], start: std::time::Instant) {
    observe(name, labels, start.elapsed().as_secs_f64());
}

#[cfg(test)]
mod tests;
