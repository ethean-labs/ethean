# Status handshake boot wiring and req/resp outboxes (2026-09-19)

## What landed

Boot path now stores a local Lean `Status`, drains QuicSwarm events, queues
`StatusSessionBook` entries for connected peer fingerprints, and stages encoded
Status payloads on `SwarmFacade.status_outbox` (request tracker ids allocated).

After a completed Status ingest, `complete_status_handshake` can stage a
blocks-by-root outbound on `SwarmFacade.blocks_outbox` when the remote head root
differs from the local head.

`client.rs` stayed under the 2000-line budget by moving boot gates / observability
finish into `client_boot.rs` and the boot pump into `client_swarm.rs`.

## Honest gaps

- QuicSwarm behaviour is still ping + gossipsub only. Outbox payloads are not
  sent on libp2p request_response streams yet.
- No public pq-devnet-5 bootnodes; operator multiaddrs + fork digest remain
  required for a live mesh join.
- leanSig / leanVM production gates remain fail-closed.

## Key modules

- `crates/node/src/status_handshake.rs`
- `crates/node/src/client_boot.rs`
- `crates/network/src/reqresp/status_outbound.rs`
- `crates/network/src/reqresp/blocks_outbound.rs`
