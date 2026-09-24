# Wall-clock duty loops wait for genesis (2026-09-24)

## Symptom

Hive's lean simulator writes a `config.yaml` whose `GENESIS_TIME` lies a few
seconds in the future, then starts the client and probes `/lean/v0`. Ethean's
first wall-clock duty step hit `ClockError::PreGenesis`, logged
`Duty step failed; shutting down`, and exited. Every `rpc-compat` test after
"client launch" failed because the HTTP listener was gone. lean-quickstart
behaves the same way when a node is started before the devnet's genesis.

## Change

`crates/node/src/wall_tick.rs` gains `ms_until_genesis(clock, now_ms)`.
`crates/node/src/duty_mesh.rs` checks it at the top of both wall-clock loops
(`run_wall_with_flush`, `one_mesh_step`): before genesis the loop keeps the
swarm pumped and the API snapshot refreshed, logs one
`waiting for genesis before running duties` line, naps for at most one
second and tries again. No tick is consumed and no duty runs until the
genesis time arrives. Nothing changes after genesis.

## Checks

- `cargo test -p ethean-node --lib wall_tick` (new `genesis_wait_only_before_genesis`).
- hive `rpc-compat` against the rebuilt image (see the hive notes of the same day).
