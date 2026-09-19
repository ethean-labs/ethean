//! Shared registry for concurrent scrape + duty-loop writers.

use crate::registry::Registry;
use std::sync::{Arc, Mutex};

/// Process-wide registry handle (`Arc` + mutex).
#[derive(Clone, Debug)]
pub struct SharedRegistry {
    inner: Arc<Mutex<Registry>>,
}

impl SharedRegistry {
    /// Wrap an existing registry.
    pub fn new(registry: Registry) -> Self {
        Self {
            inner: Arc::new(Mutex::new(registry)),
        }
    }

    /// Lock and mutate the registry.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut Registry) -> R) -> R {
        let mut guard = self.inner.lock().expect("metrics registry poisoned");
        f(&mut guard)
    }

    /// Lock and read the registry.
    pub fn with_ref<R>(&self, f: impl FnOnce(&Registry) -> R) -> R {
        let guard = self.inner.lock().expect("metrics registry poisoned");
        f(&guard)
    }
}
