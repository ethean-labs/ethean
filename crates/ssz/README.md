# ethean-ssz

Minimal Simple Serialize (SSZ) for Ethean Lean Consensus containers.

## Scope

- Little-endian unsigned integers, fixed byte arrays, booleans
- Offset-based lists of variable-size elements (basic)
- SHA-256 `hash_tree_root` for basic types and containers (chunk packing)

This crate is **Ethean-owned**. Phase 00 leanSpec uses `eth-ssz-specs` in Python; the Rust path does not depend on peer-client codec crates or `ethereum_ssz`.

## Non-goals

Full SSZ union support, Bitvector merkleization edge cases beyond Bitlist basics, and production fuzz harnesses land in later phases.
