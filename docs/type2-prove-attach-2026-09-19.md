# Type-2 prove attach on proposal path (2026-09-19)

## Shared statement

- `ethean_transition::type2_statement_for_block` builds the consensus Type-2 inputs
- `apply_block` now calls the same helper (no drift vs the node prover)

## Node prove

- `block_builder::try_attach_type2_proof` runs `ProverWorker` → `verify_type2` → writes `plan.aggregate_proof`
- Fail-closed: backend gaps / verify failure leave the plan unchanged
- `production_type2_ready` mirrors `FfiStatus::leanvm` (false until FFI linked)

## Duty step

- On `publish_allowed`: prove Type-2 → emit `Type2ProofAttached` → local proposer sign → encode gossip
- With workspace default `test-aggregate`, gossip envelopes carry a statement-bound synthetic proof
- Remotes with the same feature accept via `apply_block`; production builds without the feature still refuse fake verify

## Verification

```text
cargo test -p ethean-transition --lib type2
cargo test -p ethean-node --lib block_builder::type2
cargo test -p ethean-node --lib duty_step
cargo test -p ethean-node --lib
```

## Still open

- Upstream leanSig `num-bigint` 0.5 so git dep works without path override
- Set `LEANVM_FFI_LINKED` after real lean-multisig FFI symbols link
- Embed proposer XMSS bytes into the Type-2 proof blob when leanMultisig specifies the encoding
