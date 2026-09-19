# Networking (Lean)

P2P is owned by `ethean-network` and `ethean-network-wire`.

## Status

- Gossip encode/decode, admission limits, peer records, status exchange: present
- UDP `BoundTransport` + `probe_udp_status` for path checks (default)
- Optional `libp2p-quic`: `QuicSwarm` / `SwarmFacade::bind_quic_swarm` + `dial_quic_peer`
- Snappy / topic strings: follow Lean wire pins, not Beacon gossipsub topic reuse

Node crate re-exports the Lean network API (`ethean-node/libp2p-quic` forwards the feature).
Legacy libp2p Beacon modules were deleted.

See Phase 10 notes under `docs/lean-consensus-migration-phase-10-quic-gossip-2026-09-19.md`
and [external-gates-quic-rocksdb-leansig-2026-09-19.md](./external-gates-quic-rocksdb-leansig-2026-09-19.md).
