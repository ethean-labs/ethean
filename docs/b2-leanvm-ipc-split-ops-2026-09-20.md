# leanVM IPC SplitRequest / SplitResponse (2026-09-20)

## What landed

| Piece | Change |
| --- | --- |
| `IpcOp` | `SplitRequest = 5`, `SplitResponse = 6` |
| `IpcFrame::split_request` | Type-2 statement + Type-2 proof blob |
| `leanvm_ipc_split` | `encode_type1_leaves` / `decode_type1_leaves`, `split_ipc` / `split_ipc_at` |
| `ethean-leanvm-mock` | Structural SplitResponse (`crypto_split` stays false) |
| `type2_split` | Prefers `split_ipc` when prover env is set; else local structural |

When `ETHEAN_LEANVM_PROVER` is unset, `split_ipc` falls back to local
`split_type2_to_type1` (same structural leaves). With a prover present it
exchanges frames; the mock returns component descriptors without SNARK
decomposition.

## Honesty

- **Not** production Type-2 → Type-1 crypto split
- `Type1Leaf.crypto_split` remains `false` until a pin-trusted leanVM fills proofs
- Pool seeding via `type2_split` still uses structural leaves today

## Tests

```bash
cargo test -p ethean-crypto --lib leanvm_ipc
cargo build -p ethean-leanvm-mock
```

## Follow-ups

- Real leanVM SplitResponse with independently verifiable Type-1 proofs
- Prefer pin-trusted leanVM binary over the mock before mesh crypto claims
