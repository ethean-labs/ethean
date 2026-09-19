# leanVM statement wire and process-IPC gate (2026-09-19)

## Goal

Move leanVM toward Phase 08 process isolation without linking Plonky3 into the consensus process or claiming readiness early.

## Landed

- `AggregateStatement::encode_wire` / `decode_wire` — canonical public-input bytes (same layout as `digest`).
- `leanvm_ipc` — probes `ETHEAN_LEANVM_PROVER`, never spawns; `protocol_ready` stays false until a framed ABI ships.
- `backend_leanvm` prove/verify routes through IPC when in-process `LEANVM_FFI_LINKED` is false.
- `LeanVmGate` exposes `ipc_binary_present` / `ipc_protocol_ready`; `ready()` needs feature plus FFI **or** IPC ready.
- Boot logs include `leanvm_ipc_binary` and `leanvm_ipc_ready`.
- `tools/release/check-leanvm-prover.ps1` checks pin + modules + optional env path.

## Still fail-closed

| Condition | Behaviour |
| --- | --- |
| No prover env | prove/verify error |
| Env set, file missing | gate `ipc_binary_present=false` |
| File present | still refuse spawn (`protocol_ready=false`) |
| `LEANVM_FFI_LINKED=true` without symbols | second error path |

## Next

1. Define versioned request/response frames (statement wire + proof blob + pin).
2. Spawn sandbox with CPU/memory/wall deadlines; kill on wedge.
3. Flip `protocol_ready` only after round-trip tests against pin `e2592df4…`.
4. Upstream leanSig `num-bigint` 0.5; embed proposer XMSS into Type-2 when leanMultisig specifies encoding.
