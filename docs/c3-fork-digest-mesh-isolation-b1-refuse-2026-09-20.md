# Fork-digest mesh isolation warnings (C3) and leanSig refuse_reason (B1)

Date: 2026-09-20

## What landed

### C3 honesty

- New module `crates/node/src/fork_digest_policy.rs`:
  - `ForkDigestSource::{OperatorOverride, InterimNameHash}`
  - `NetworkTarget::fork_digest_source()` / `mesh_isolation_risk()`
- `mesh_isolation_risk` is true only for `pq-devnet-4` / `pq-devnet-5` when bootnodes are set and no operator `--fork-digest` (env / file) is present.
- `client_boot::boot_gates` logs the resolved fork segment source and **warns** when dialing would isolate Lean gossip topics from a peer mesh.

### B1 honesty

- `LeanSigGate::refuse_reason()` returns a static string when `leansig-backend` is off.
- Boot logs that reason when dialing bootnodes without production leanSig (peer XMSS verify stays fail-closed).

### Build hygiene

- `client_swarm` imports gated on `libp2p-quic`.
- Non-QUIC `apply_network_budget` touches `status_sessions` to avoid dead_code.
- `duty_mesh` allows `unused_mut` when the pending-event push path is cfg'd out.

## Still open

- Operator-provided fork digest **values** for a live D5 run (A2 value TBD / A3 bootnodes).
- Production `leansig-backend` link + verify path (real B1 close).
- Type-2 SNARK verify (B2/B3) beyond the empty-proof envelope gate (C4).

## Tests

- `cargo test -p ethean-node --lib fork_digest --features libp2p-quic`
- `cargo test -p ethean-crypto --lib default_build_reports_gaps`
