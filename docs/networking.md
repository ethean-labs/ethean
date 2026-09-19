# Networking (Lean)

P2P is owned by `ethean-network` and `ethean-network-wire`.

## Status

- Gossip encode/decode, admission limits, peer records, status exchange: present
- QUIC transport: open gate (`TransportPending` until wired)
- Snappy / topic strings: follow Lean wire pins, not Beacon gossipsub topic reuse

Node crate re-exports the Lean network API; legacy libp2p Beacon modules were deleted.

See Phase 10 notes under `docs/lean-consensus-migration-phase-10-quic-gossip-2026-09-19.md`.
