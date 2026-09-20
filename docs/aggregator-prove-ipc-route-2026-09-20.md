# Aggregator prove routes to leanVM IPC when prover binary is set (2026-09-20)

## What landed

`prove_type1` / `prove_type2` / verify now prefer process IPC when
`ETHEAN_LEANVM_PROVER` points at an existing file:

1. FFI (still unlinked)
2. Process IPC via `prove_ipc` / `verify_ipc`
3. `test-aggregate` smoke bindings (default offline path)

Duty aggregator (`ProverWorker` → `try_prove_type1_for_root`) and Type-2 attach
pick this up automatically — no separate duty-loop fork.

## Anti-recursion

Spawned children get:

- `ETHEAN_LEANVM_PROVER` / `ETHEAN_LEANVM_IPC_PROBE` removed
- `ETHEAN_LEANVM_IPC_SERVING=1` set

While serving, `prove_bound` / `verify_bound` skip IPC and use local bindings so
`ethean-leanvm-mock` (which calls `prove_type1`) cannot spawn itself forever.
The mock also clears/sets those env vars at process start.

## Operator recipe

```powershell
cargo build -p ethean-leanvm-mock
$env:ETHEAN_LEANVM_PROVER = "$PWD\target\debug\ethean-leanvm-mock.exe"
# optional gate green:
$env:ETHEAN_LEANVM_IPC_PROBE = "1"
ethean start --network pq-devnet-4
```

## Honesty

- Mock is still not production leanVM / Plonky3
- Without `ETHEAN_LEANVM_PROVER`, behaviour is unchanged (test-aggregate)
- Real leanVM binary remains the production target

## Still open

| Gap | Notes |
| --- | --- |
| Real leanVM in CI | Prefer over mock for production claims |
| Blocks-by-range QuicSwarm stream | Next mesh sync item |
| A2/A3 fork-digest + bootnodes | Operator values |
