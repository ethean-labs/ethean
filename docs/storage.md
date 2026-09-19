# Storage (Lean)

Owned by `ethean-storage`.

- Schema id / version: `ethean-lc-d5-v1` (see crate `schema` module)
- In-memory `Database` with checksummed puts and flush-before-publish batching
- Path open: `open_path` refuses legacy markers, then returns `BackendPending` until RocksDB is bound
- Optional feature `ethean-storage/rocksdb` (no silent memory fallback when pending)
- Legacy path refusal for `panro` markers and `.json` consensus roots

Sync/checkpoint trust lives in `ethean-sync`. Node client opens the Lean in-memory DB on smoke start.
