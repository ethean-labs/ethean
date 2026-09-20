# B2 leanVM IPC frames and B4 aggregation pin checks (2026-09-20)

## B2 — IPC frame ABI

Landed versioned leanVM process-IPC frames without enabling spawn:

- Module `crates/crypto/src/leanvm_ipc_frame.rs`
  - Magic `ELVM`, `FRAME_VERSION = 1`
  - Ops: ProveRequest / ProveResponse / VerifyRequest / VerifyResponse
  - Carries 40-hex `LEANVM_REV` pin + statement wire + proof blob
- `LeanVmIpcStatus.frame_abi_ready` is true when the codec is compiled
- `protocol_ready` stays **false** (no process spawn / round-trip yet)
- `prove_ipc` / `verify_ipc` encode a real frame, then refuse spawn
- `LeanVmGate.ipc_frame_abi_ready` logged at boot as `leanvm_ipc_frame`

## B4 — operator aggregation pin

- `crates/node/src/agg_pin.rs` parses `KEY=value` from
  `config/networks/{pq-devnet-4|5}.aggpin`
- Compared fields: `LOG_INV_RATE`, `LEANVM_REV`, `LEANSIG_REV`
- Boot warns on mismatch; info-logs when a present pin matches this build
- Template files committed with current local constants (operators overwrite)

## Still fail-closed / open

| Gap | Status |
| --- | --- |
| leanVM spawn + sandbox deadlines | Open (`protocol_ready=false`) |
| Production leanSig XMSS (`leansig-backend` + vendor patch) | Local compile OK; git dep still blocked (B1 residual) |
| Production leanVM SNARK prove/verify | Open (B2/B3) |
| Real `split_type_2` crypto bytes | Open |

## Tests

- `cargo test -p ethean-crypto --lib leanvm_ipc`
- `cargo test -p ethean-node --lib agg_pin`
- `cargo test -p ethean-node --lib` (79 passed)
