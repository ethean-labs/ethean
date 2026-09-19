# Security (Lean)

- Production crypto is XMSS / leanSig oriented via `ethean-crypto` (fail-closed without pinned FFI).
- Aggregation proofs go through leanVM-facing APIs (fail-closed prove until FFI lands).
- No BLS crates remain in the workspace dependency graph.
- Signer watermarks must not rewind on chain rollback (storage watermark field).
- Soft legacy scans reject `panro`, `blst`, Beacon `/eth/v1` surfaces outside allowlisted retirement docs.

Release gates: [release/gates.md](./release/gates.md).
