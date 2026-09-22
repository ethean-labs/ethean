# Committed leanSpec-shaped samples

Tiny JSON envelopes for CI without the ~154 MiB production archive.

| Sample | leanSpec rejection |
| --- | --- |
| [`fork_choice/block_beyond_future_horizon_rejected.json`](./fork_choice/block_beyond_future_horizon_rejected.json) | `BLOCK_TOO_FAR_IN_FUTURE` |

Full filled vectors: run `tools/release/fetch-leanspec-fixtures.sh`, then set
`ETHEAN_LEANSPEC_FIXTURES` to the extract root. See
[`../phase-00/README.md`](../phase-00/README.md) and
[`../../../docs/hive-leanspec-fixture-consumer-2026-09-20.md`](../../../docs/hive-testing/hive-leanspec-fixture-consumer-2026-09-20.md).
