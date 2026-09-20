# Type-2 structural split and pool reseed (2026-09-19)

## What landed

pq-devnet-5 needs Type-2 block proofs to be **decomposable** into per-message Type-1
leaves so the next proposer can extend `known_aggregated_payloads`.

1. **`ethean-crypto` `aggregation/split.rs`**
   - `split_type2_to_type1` / `attestation_leaves_from_type2`
   - Structural component rows only; `crypto_split = false` and empty proof bytes until
     leanVM IPC exposes a real SNARK split.

2. **`ethean-node` `aggregation/type2_split.rs`**
   - `seed_pool_from_signed_block` inserts pool entries keyed by attestation-data root,
     carrying body attestation SSZ + coverage.

3. **`gossip_stf`**
   - After `AppliedVerified` (and `RootOnly` with a non-empty Type-2 proof), reseeds the
     aggregate pool from the imported `SignedBlock`.

## Why

Interop #37 / leanSpec #717: folding to a single block proof without recoverable
intermediates breaks the next proposer's aggregation cache. This lands the cache
recovery path structurally while leanVM split stays fail-closed.

## Still open

- Real leanVM `split_type_2` proof bytes (`crypto_split = true`).
- Type-1 merge IPC (B2) and production leanSig verify (B1).
- Operator bootnodes / fork digest for live mesh join.
