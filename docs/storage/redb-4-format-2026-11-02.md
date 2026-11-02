# redb 4.3 after Dependabot merge (2026-11-02)

`ethean-labs/ethean` #7 merged `redb` 2.6.3 → 4.3.0 onto `master` while CI was
red. Compile recovery: `begin_read` moved onto the `ReadableDatabase` trait.

## File format

redb 4 expects file format version 3. An `ethean.redb` written by 2.x is version
1 or 2. `Database::open` then returns `UpgradeRequired`; there is no in-place
upgrade.

Opening such a file now fails with a config error that names the path and tells
the operator to delete `ethean.redb` or `--reset-chain`. SSZ files under
`--data-dir` are unchanged; only the KV file is incompatible.

Fresh `--data-dir` (or a wipe) creates a v3 file. Do not copy a pre-0.1.72
`ethean.redb` onto a node built against redb 4.
