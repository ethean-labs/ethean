# B2 leanVM IPC live probe flips protocol_ready (2026-09-20)

## What landed

- Env `ETHEAN_LEANVM_IPC_PROBE` (`1`/`true`/`yes`/`on`)
- When set **and** `ETHEAN_LEANVM_PROVER` points at an existing binary,
  `LeanVmIpcStatus::probe` runs one pin-checked Type-1 prove round-trip
- On success: `protocol_ready = true` (boot / `LeanVmGate.ready` can go green)
- Default remains fail-closed (no probe → `protocol_ready` stays false)
- Boot log field `leanvm_ipc_probe`
- Refuse reason tells operators to set the probe env when a binary is present
- Tests moved to `leanvm_ipc_tests.rs` (file-size cap); env tests serialized

## Operator recipe

```powershell
cargo build -p ethean-leanvm-mock
$env:ETHEAN_LEANVM_PROVER = "$PWD\target\debug\ethean-leanvm-mock.exe"
$env:ETHEAN_LEANVM_IPC_PROBE = "1"
# boot / LeanVmGate::probe() now reports ipc_protocol_ready when the mock answers
```

## Honesty

- Mock still is **not** production leanVM / Plonky3 SNARK
- Ops should point `ETHEAN_LEANVM_PROVER` at a pin-trusted leanVM binary before
  treating IPC as production-ready
- In-process FFI (`LEANVM_FFI_LINKED`) remains unlinked

## Still open

| Gap | Notes |
| --- | --- |
| Real leanVM binary in CI | Preferred over mock for `protocol_ready` claims |
| Aggregator path using `prove_ipc` when gate ready | Next wiring |
| B1 upstream leanSig `num-bigint` 0.5 | Git dep still blocked |
