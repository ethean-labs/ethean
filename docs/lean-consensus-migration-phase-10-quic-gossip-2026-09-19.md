# Phase 10 — QUIC, gossip, and req/resp (2026-09-19)

## Summary

Phase 10 delivers wire codecs and network application validation; QUIC transport remains a fail-closed facade.

### ethean-network-wire
- Topics `/leanconsensus/{fork}/…/ssz_snappy` (never `12345678`)
- Raw Snappy (gossip) + framed Snappy (req/resp) with bomb/trailing guards
- Status encode/decode + genesis/fork compatibility
- Blocks-by-root / range requests capped at 1024

### ethean-network
- Admission caps, `NodeIdentity`, `PeerManager`
- Gossip ACCEPT/IGNORE/REJECT (rejects `/eth2/`)
- Status handler + request tracker
- `prepare_transport` / dial fail closed (no TCP/WS)

## Tests
- `ethean-network-wire`: **10 passed**
- `ethean-network`: **8 passed**

## Open gates
libp2p QUIC-v1 swarm, Phase 00 `fork_identifier_bytes` / message-ID preimage order, ENR discovery, 72h soak.
