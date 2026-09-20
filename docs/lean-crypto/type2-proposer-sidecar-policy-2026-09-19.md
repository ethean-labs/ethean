# Type-2 proposer Sidecar policy (2026-09-19)

## Goal

Keep local proposer XMSS off `SignedBlock.proof` until leanMultisig pins how (or whether) raw signatures merge into the Type-2 blob.

## Landed

- `block_builder/type2_envelope.rs`: `PROPOSER_TYPE2_POLICY = Sidecar`, `wire_proof_bytes`, `assert_sidecar_invariant`.
- `assemble_signed_block` runs the invariant and uses `wire_proof_bytes` before SSZ encode.
- Accidental concatenation of the proposer signature into the proof is rejected.

## Why

Remote `apply_block` fail-closes on proof bytes that are not a leanVM aggregate. Stuffing HMAC/XMSS into the proof field would break ingest.

## Next

Flip policy only when leanMultisig documents the encoding; then extend `wire_proof_bytes` and re-verify with production leanVM/IPC before gossip.
