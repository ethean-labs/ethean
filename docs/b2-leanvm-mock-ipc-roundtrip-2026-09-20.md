# B2 ELVM mock prover and IPC round-trip (2026-09-20)

## What landed

1. **`bin/ethean-leanvm-mock`**
   - Speaks length-prefixed ELVM on stdio
   - ProveRequest → `prove_type1` / `prove_type2` (test-aggregate bindings)
   - VerifyRequest → statement-bound verify
   - Pin-checks `LEANVM_REV`
2. **`read_len_prefixed`** exported from `ethean-crypto` for mock / peers
3. **`try_roundtrip_prove(path)`** — one Type-1 prove exchange without flipping `protocol_ready`
4. Unit test `roundtrip_against_workspace_mock_if_built` (runs when mock is in `target/debug`)

## How to use locally

```powershell
cargo build -p ethean-leanvm-mock
$env:ETHEAN_LEANVM_PROVER = "$PWD\target\debug\ethean-leanvm-mock.exe"
cargo test -p ethean-crypto --lib roundtrip_against_workspace_mock
```

## Honesty

- Mock is **not** production leanVM / Plonky3
- `LeanVmIpcStatus.protocol_ready` stays **false** until a real leanVM pin round-trip is trusted in ops/CI
- Spawn path is now end-to-end proven against an ELVM peer

## Still open

- Real leanVM binary + CI green → flip `protocol_ready`
- B1 production `leansig-backend` vendor path
