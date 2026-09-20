# Soft-skip gossip InsufficientPeers under local finality (2026-09-20)

## Symptom

Solo `--local-finality` applied a block (`Local finality applied slot=…`), then the
duty loop exited:

```text
Duty step failed; shutting down
error=Network error: handshake: gossip publish: InsufficientPeers
```

Head stopped advancing; `:9100` became unreachable once the process exited.

## Cause

After local apply, `publish_pending_block` still tried Gossipsub publish on an
empty mesh. libp2p returned `InsufficientPeers`, mapped to `NetworkError::Handshake`,
and `duty_mesh` treated any flush error as fatal.

## Fix

In `crates/node/src/swarm_pump.rs`, when `owner.local_finality` is set and the
publish error message contains `InsufficientPeers`, treat publish as a soft skip:
keep block bytes in the facade store, return `Ok(PublishedBlock)`, do not restore
pending gossip (already applied locally). Other publish errors stay fatal.

## Verify

```powershell
ethean start --until-signal --network pq-devnet-4 --local-finality
# wait ~20s, then:
curl -s http://127.0.0.1:9100/metrics | findstr ethean_head_slot
```

`ethean_head_slot` should climb without Docker/WSL or public bootnodes.
