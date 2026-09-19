//! Durable Lean storage: schema, batches, tables, prune, recovery.

#![forbid(unsafe_code)]

pub mod batch;
pub mod db;
pub mod error;
pub mod prune;
pub mod recovery;
pub mod rocks;
pub mod schema;
pub mod tables;

pub use batch::{apply_puts, BatchPut, WriteBatch};
pub use db::Database;
pub use error::{Result, StorageError};
pub use prune::PrunePolicy;
pub use recovery::recover_on_open;
pub use rocks::open_path;
pub use schema::{assert_schema, SCHEMA_ID, SCHEMA_VERSION};
pub use tables::{
    root_key, slot_key, META_FINALIZED, META_HEAD, META_JUSTIFIED, TABLE_BLOCKS, TABLE_METADATA,
    TABLE_PARENT, TABLE_POOLS, TABLE_SLOT_ROOT, TABLE_STATES, TABLE_VOTES,
};
