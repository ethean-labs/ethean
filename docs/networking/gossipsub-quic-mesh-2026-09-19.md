# Gossipsub mesh on QuicSwarm (2026-09-19)

## What landed

- `LeanGossipTopics` builds `/leanconsensus/{fork}/block|aggregation|attestation_0/ssz_snappy`
- Feature `libp2p-quic` now enables `libp2p/gossipsub`
- `QuicSwarm::bind_for_fork` subscribes the mesh; `publish_gossip` refuses `/eth2/`
- Inbound messages run through `validate_gossip_payload` during `pump_once`
- `SwarmFacade::{bind_quic_swarm_for_fork, publish_gossip, pump_quic_once}`
- Node boot uses profile fork name; `pump_network` drains a small event budget after boot
- `swarm_pump::pump_swarm_budget` for non-blocking duty-loop integration

## Verification

```text
cargo test -p ethean-network --features libp2p-quic --lib
cargo test -p ethean-node --features libp2p-quic --lib swarm_pump
cargo check -p ethean-node --features libp2p-quic
```

## Still open

- Consensus import path for `gossip_accept` payloads (chain owner commands)
- Peer scoring feedback wired from pump actions
- leanSig git dep / leanVM FFI (unchanged external blockers)
