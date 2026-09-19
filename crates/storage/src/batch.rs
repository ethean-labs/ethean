//! Atomic write batch with explicit flush barrier.

use ethean_primitives::Hash32;
use std::collections::HashMap;

/// One pending put of opaque SSZ (or versioned local) bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchPut {
    /// Column / table name.
    pub table: &'static str,
    /// Key bytes.
    pub key: Vec<u8>,
    /// Value bytes (canonical SSZ preferred).
    pub value: Vec<u8>,
    /// SHA-256 of value for integrity.
    pub checksum: Hash32,
}

/// Mutable batch accumulated before a single flush.
#[derive(Debug, Default)]
pub struct WriteBatch {
    puts: Vec<BatchPut>,
    flushed: bool,
}

impl WriteBatch {
    /// Stage a put.
    pub fn put(&mut self, put: BatchPut) {
        self.flushed = false;
        self.puts.push(put);
    }

    /// Number of staged puts.
    pub fn len(&self) -> usize {
        self.puts.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.puts.is_empty()
    }

    /// Drain puts for the database apply path.
    pub fn take_puts(&mut self) -> Vec<BatchPut> {
        std::mem::take(&mut self.puts)
    }

    /// Mark durable after fsync / backend flush.
    pub fn mark_flushed(&mut self) {
        self.flushed = true;
    }

    /// Whether the last apply was marked durable.
    pub fn is_flushed(&self) -> bool {
        self.flushed
    }
}

/// Apply puts into an in-memory map (test / process-local backend).
pub fn apply_puts(map: &mut HashMap<(String, Vec<u8>), (Vec<u8>, Hash32)>, puts: &[BatchPut]) {
    for p in puts {
        map.insert(
            (p.table.to_string(), p.key.clone()),
            (p.value.clone(), p.checksum),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_starts_unflushed() {
        let mut b = WriteBatch::default();
        b.put(BatchPut {
            table: "blocks",
            key: vec![1],
            value: vec![2],
            checksum: [0u8; 32],
        });
        assert!(!b.is_flushed());
        b.mark_flushed();
        assert!(b.is_flushed());
    }
}
