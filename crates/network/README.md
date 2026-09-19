# ethean-network

Lean Consensus P2P runtime scaffolding for Phase 10.

- Gossip: raw Snappy validation with ACCEPT/IGNORE/REJECT (no `/eth2/` topics)
- Req/resp: Status handshake + request tracker
- Transport: UDP listen **bind** via `prepare_transport` → `BoundTransport`; TCP/WS refused
- `probe_udp_status`: UDP Status path probe (reachability; not TLS/QUIC crypto)
- Dial / libp2p QUIC-v1 swarm still `TransportPending` via `dial_quic`
- Peer admission caps and disposable scores
