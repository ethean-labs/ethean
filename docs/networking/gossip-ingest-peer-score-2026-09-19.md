# Gossip ingest and peer score feedback (2026-09-19)

## Peer scoring

- `QuicSwarm::pump_once` returns structured [`PumpEvent`] / [`GossipIngress`]
- `SwarmFacade::pump_quic_once` applies `delta_for(action)` via `PeerManager::ensure_and_feedback`
- Propagation source PeerId is fingerprinted with SHA-256

## Chain ingest

- `ChainCommand::IngestGossip` + `ChainEvent::GossipIngested`
- Content root = SHA-256(`ethean-gossip-ingest-v1` || payload); head is **not** advanced
- `ChainOwner.last_gossip_root` stores the provisional id until SSZ block import lands
- `gossip_ingest::ingest_accepted` turns ACCEPT payloads into commands
- Boot pump collects accepted gossip and ingests before duty loops

## Layout

- `boot_network` extracts UDP/QuicSwarm bind from `client.rs` (≤2000 line rule)
- `swarm_pump::PumpBudgetResult` carries `drained` + `accepted`

## Verification

```text
cargo test -p ethean-network --features libp2p-quic --lib
cargo test -p ethean-node --features libp2p-quic --lib
```

## Still open

- Attestation-subnet SSZ into duty/pool paths; full state transition on import
- leanSig git dep / leanVM FFI
