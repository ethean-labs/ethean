//! Durable persist / range-serve metric helpers.

use crate::error::Result;
use crate::registry::Registry;

/// Accrue durable flush and prune counts after a successful data-dir save.
pub fn record_durable_persist(
    reg: &mut Registry,
    flushed_blocks: u64,
    floor_slot: u64,
    files_removed: u64,
    redb_removed: u64,
    keep_slots: u64,
) -> Result<()> {
    if flushed_blocks > 0 {
        reg.inc("durable_blocks_flushed_total", flushed_blocks as f64)?;
    }
    if files_removed > 0 {
        reg.inc("durable_blocks_pruned_files_total", files_removed as f64)?;
    }
    if redb_removed > 0 {
        reg.inc("durable_blocks_pruned_redb_total", redb_removed as f64)?;
    }
    reg.set("durable_blocks_prune_floor_slot", floor_slot as f64)?;
    reg.set("durable_blocks_prune_keep_slots", keep_slots as f64)?;
    Ok(())
}

/// Mirror blocks-by-range serve totals from the QuicSwarm atomics.
pub fn record_range_serve(
    reg: &mut Registry,
    found_total: u64,
    missing_total: u64,
    cache_slots: u64,
) -> Result<()> {
    reg.set("blocks_by_range_serve_found_total", found_total as f64)?;
    reg.set("blocks_by_range_serve_missing_total", missing_total as f64)?;
    reg.set("blocks_by_range_serve_cache_slots", cache_slots as f64)?;
    Ok(())
}

/// Record boot-time durable serve-cache seed counts.
pub fn record_serve_cache_seed(reg: &mut Registry, candidates: u64, indexed: u64) -> Result<()> {
    reg.set("serve_cache_seed_candidates", candidates as f64)?;
    reg.set("serve_cache_seed_indexed", indexed as f64)?;
    Ok(())
}
