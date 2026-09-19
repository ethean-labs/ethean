# ethean-network

Lean Consensus P2P runtime scaffolding for Phase 10.

- Gossip: raw Snappy validation with ACCEPT/IGNORE/REJECT (no `/eth2/` topics)
- Req/resp: Status handshake + request tracker
- Transport: QUIC-v1 facade **fails closed**; `reject_non_quic` refuses TCP/WS/WSS
- Peer admission caps and disposable scores

Full libp2p QUIC swarm wiring remains an open gate.
