# D3 Type-2 from Type-1 cache (2026-09-20)

## What landed

Proposer path now builds the Type-2 block envelope from the **Type-1 aggregate cache**, not from a stolen Type-1 proof blob:

1. **`body_from_pool`** prefers pool entries that already carry a non-empty Type-1 proof, then fills with SSZ-only rows.
2. **`plan_from_pool`** leaves `aggregate_proof` empty until attach (no more copying a Type-1 proof into the Type-2 field).
3. **`try_attach_type2_from_type1_cache`** (new `type2_cache` module):
   - Counts Type-1 hits vs packed body attestations.
   - Mesh path (`require_type1_cache = true`, i.e. not local-finality): skips Type-2 prove until every packed attestation has a pool Type-1 proof.
   - Local-finality smoke still proves without cache (empty/injected bodies).
4. **`Type2ProofAttached`** now reports `type1_hits` / `type1_needed`.
5. Proposal logic lives in **`duty_propose`** so `duty_step` stays ≤300 lines.

## Flow

```text
Type-1 pool (gossip / AggregatorType1Proved)
  → pack body (proved first)
  → if mesh: require full Type-1 coverage
  → prove/verify Type-2 statement for block
  → SignedBlock.proof + /block/ gossip
```

## Still open

- Production leanVM SNARK prove (B2/B3) — test-aggregate still supplies synthetic proofs in default builds.
- Real leanVM `split_type_2` proof bytes back into the cache (`crypto_split = true`).
- Operator bootnodes / fork digest for a live D5 mesh.

## Tests

- `cargo test -p ethean-node --lib type2_cache`
- `cargo test -p ethean-node --lib prefers_type1`
- `cargo test -p ethean-node --lib duty_propose`
- `cargo test -p ethean-node --lib` (77 passed)
