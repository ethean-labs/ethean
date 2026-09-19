# QuicSwarm Lean Status request_response wire (2026-09-19)

## What landed

QuicSwarm behaviour is now ping + gossipsub + Lean Status
`request_response` using framed Snappy on
`/leanconsensus/req/status/1/ssz_snappy`.

Boot path:

1. Cache local Status SSZ on the swarm for inbound replies.
2. Queue `StatusSessionBook` for connected peers.
3. Stage + `flush_status_outbox` so outbound Status requests leave on QUIC
   streams when the peer fingerprint is still connected.

Pump events include `StatusRequest` / `StatusResponse` with decompressed SSZ.

## Still open

- Completing handshakes from `StatusResponse` into `complete_status_handshake`
  during the duty loop (ingest + blocks-by-root flush).
- Blocks-by-root still stages on the facade outbox only; no stream codec yet.
- Operator bootnodes + fork digest still required for a live pq-devnet-5 join.
