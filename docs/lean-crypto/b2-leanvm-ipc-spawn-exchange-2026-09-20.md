# B2 leanVM IPC spawn exchange (2026-09-20)

## What landed

Process isolation scaffolding for Phase 08 leanVM prove/verify:

1. **`leanvm_ipc_spawn`**
   - Spawns `ETHEAN_LEANVM_PROVER`
   - Writes one **u32-LE length-prefixed** ELVM request frame to stdin
   - Reads one length-prefixed response from stdout
   - **Wall deadline** (default 30s) kills the child on wedge
2. **`prove_ipc` / `verify_ipc`**
   - Build ELVM request → `exchange_frame` → pin/op/statement checks
   - Fail-closed on missing binary, non-prover peers, pin mismatch, empty proof
3. **Gate honesty**
   - `spawn_exchange_wired = true` (code path exists)
   - `protocol_ready` stays **false** until a live pin-checked round-trip against leanVM `e2592df4…` is green in ops/CI
   - Boot logs `leanvm_ipc_spawn`

## Still open

- Real leanVM binary that speaks ELVM + length-prefix
- Flip `protocol_ready` after CI round-trip
- Sandbox resource limits beyond wall time (CPU/memory cgroups)
- Production `leansig-backend` vendor path (B1)

## Tests

- `cargo test -p ethean-crypto --lib leanvm_ipc` (9 passed)
- Non-prover Windows `where.exe` spawn fails closed
- `cargo test -p ethean-node --lib` (79 passed)
