# Interop tests

Cross-crate and peer-differential cases. Phase fixtures under `spec/fixtures/` are the authority.

## leanSpec / Hive fixture consumer

- Crate: `ethean-spec-fixtures` (envelope parse + rejection mapping + discovery)
- Fetch: `tools/release/fetch-leanspec-fixtures.sh` → `.cache/leanspec-fixtures/`
- Env: `ETHEAN_LEANSPEC_FIXTURES` = extract root
- Notes: [`../../docs/hive-leanspec-fixture-consumer-2026-09-20.md`](../../docs/hive-testing/hive-leanspec-fixture-consumer-2026-09-20.md)

```bash
cargo test -p ethean-spec-fixtures
```
