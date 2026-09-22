//! Minimal scoped-thread fan-out; keeps the crate free of runtime dependencies.

/// Number of worker threads to use for CPU-bound fan-out.
pub fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Map `items` to outputs in order, splitting the work into contiguous
/// chunks across `worker_count()` threads.
pub fn map_parallel<T, U, F>(items: &[T], f: F) -> Vec<U>
where
    T: Sync,
    U: Send,
    F: Fn(&T) -> U + Sync,
{
    let workers = worker_count().min(items.len().max(1));
    if workers <= 1 || items.len() < 2 {
        return items.iter().map(&f).collect();
    }
    let chunk = items.len().div_ceil(workers);
    let mut results: Vec<Vec<U>> = Vec::with_capacity(workers);
    std::thread::scope(|scope| {
        let handles: Vec<_> = items
            .chunks(chunk)
            .map(|slice| scope.spawn(|| slice.iter().map(&f).collect::<Vec<U>>()))
            .collect();
        for handle in handles {
            results.push(handle.join().expect("parallel worker panicked"));
        }
    });
    results.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_order() {
        let items: Vec<u64> = (0..1000).collect();
        let out = map_parallel(&items, |x| x * 2);
        assert_eq!(out, items.iter().map(|x| x * 2).collect::<Vec<_>>());
        assert!(map_parallel::<u64, u64, _>(&[], |x| *x).is_empty());
    }
}
