# Type-1 recursive merge (2026-09-19)

## What landed

pq-devnet-5 aggregators must coalesce multiple Type-1 proofs for the same
`(message, slot)` before the proposer builds a Type-2 block proof.

1. **`ethean-crypto` `aggregation/merge.rs`**
   - `merge_type1_statements` — union participant sets (strictly increasing)
   - `merge_type1` — verify both proofs, merge statement, re-prove via `prove_type1`
     (leanVM fail-closed; `test-aggregate` for unit tests)

2. **`ethean-node` `aggregation/pool.rs`**
   - `variants(&key)` — expose all retained proofs for a message

3. **`ethean-node` `aggregation/merge_pool.rs`**
   - `merge_best_pool_variants` — merge the two highest-coverage variants

4. **`gossip_pool`**
   - After inserting a signed attestation aggregate, attempt a pool merge

## Why

Devnet-4/5 aggregator role: recursive merge of partial Type-1 aggregates over the
same attestation data. Without this path, the pool only stores variants side-by-side.

## Still open

- Production leanVM prove for merge (B2 crypto gate)
- leanSig production sign/verify (B1)
- leanVM Type-2 SNARK split (`crypto_split = true`)
