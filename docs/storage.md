# Storage (Lean)

Owned by `ethean-storage`.

- Schema id / version: `ethean-lc-d5-v1` (see crate `schema` module)
- In-memory `Database` with checksummed puts and flush-before-publish batching
- RocksDB backend: deferred (workspace pin retained for the future feature)
- Legacy path refusal for `panro` markers and `.json` consensus roots

Sync/checkpoint trust lives in `ethean-sync`. Node client opens the Lean DB on smoke start.
