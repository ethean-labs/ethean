# Aggregator and gossip decode copy cuts (2026-11-02)

Follows [copy-cuts-gossip-cap-2026-11-02.md](./copy-cuts-gossip-cap-2026-11-02.md).

## Head state

Aggregator interval work cloned the whole head `State` (validator registry
and justifications) twice: once to decide whether to run, once for union
merges. Public keys are now read from `owner.head_state.as_ref()` for the
duration of each lookup, then the prover job is submitted.

## Pool

`AggregatePool::variants` returns a slice. Child selection and block-body
candidates clone only the proofs they keep. Pool metrics decode
attestation SSZ from `best()` instead of cloning every best `PoolEntry`.

## Gossip decode

`DecodedBlockGossip` no longer duplicates `signed.block`. Every ingest used
to clone the inner block even when the import was skipped.

## Tests

Existing `aggregation_duty`, `gossip_decode`, `gossip_stf`, and pool tests.
