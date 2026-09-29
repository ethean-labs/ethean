//! Background XMSS window preparation for installed registry keys.
//!
//! A PROD bottom tree covers 65536 slots and takes seconds to build on all
//! cores. Left to the signing path, the build would run under the key lock
//! at the moment a duty needs a signature. Once a slot enters the right half
//! of a key's window, the next tree is built on a worker thread instead.

use ethean_crypto::SecretKeyMaterial;
use std::thread::JoinHandle;

/// Owns the keys to keep prepared and at most one worker thread.
#[derive(Debug, Default)]
pub struct KeyPreparer {
    keys: Vec<SecretKeyMaterial>,
    worker: Option<JoinHandle<Result<usize, String>>>,
}

impl KeyPreparer {
    /// Track a key; clones share the decoded key with the signer.
    pub fn track(&mut self, key: SecretKeyMaterial) {
        self.keys.push(key);
    }

    /// Number of tracked keys.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// True when no key is tracked.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// True while a worker thread is building trees.
    pub fn busy(&self) -> bool {
        self.worker.as_ref().is_some_and(|w| !w.is_finished())
    }

    /// Reap a finished worker and start a new one when any key needs a tree
    /// for `slot`. Cheap when nothing is due (one lock per key).
    pub fn on_slot(&mut self, slot: u64) {
        if self.busy() {
            return;
        }
        if let Some(done) = self.worker.take() {
            match done.join() {
                Ok(Ok(trees)) => tracing::info!(trees, "XMSS bottom trees prepared ahead"),
                Ok(Err(error)) => tracing::warn!(%error, "XMSS window preparation failed"),
                Err(_) => tracing::warn!("XMSS window preparation thread panicked"),
            }
        }
        let Ok(epoch) = u32::try_from(slot) else {
            return;
        };
        let due: Vec<SecretKeyMaterial> = self
            .keys
            .iter()
            .filter(|k| k.needs_prepare_ahead(epoch))
            .cloned()
            .collect();
        if due.is_empty() {
            return;
        }
        tracing::info!(
            slot,
            keys = due.len(),
            "building next XMSS bottom tree in the background"
        );
        let workers = background_workers();
        let spawned = std::thread::Builder::new()
            .name("xmss-prepare".into())
            .spawn(move || {
                let mut trees = 0;
                for key in &due {
                    trees += key
                        .prepare_ahead(epoch, workers)
                        .map_err(|e| e.to_string())?;
                }
                Ok(trees)
            });
        match spawned {
            Ok(handle) => self.worker = Some(handle),
            Err(error) => tracing::warn!(%error, "could not start XMSS preparation thread"),
        }
    }
}

/// A quarter of the cores: the next tree is due 65536 slots after the build
/// starts, so there is no hurry, while the prover and signing need the CPU now.
fn background_workers() -> usize {
    std::thread::available_parallelism()
        .map(|n| (n.get() / 4).max(1))
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_preparer_never_spawns() {
        let mut prep = KeyPreparer::default();
        prep.on_slot(1 << 20);
        assert!(!prep.busy());
        assert!(prep.is_empty());
    }
}
