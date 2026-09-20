# D2 Type-1 aggregation gossip publish (2026-09-20)

## What landed

Closes the remaining D2 wire gap after `AggregatorReady` + Type-1 prove:

1. **`aggregation_gossip`** encodes `SignedAggregatedAttestation` and queues two topics:
   - `/leanconsensus/{fork}/aggregation/ssz_snappy`
   - `/leanconsensus/{fork}/attestation_{subnet}/ssz_snappy`
2. **`ChainOwner.pending_aggregation_gossip`** holds payloads until QuicSwarm flush.
3. **`swarm_pump_agg`** publishes queued Type-1 aggregates (soft skip on `InsufficientPeers` when local-finality).
4. **`duty_step`** queues gossip after `AggregatorType1Proved`; emits `AggregationGossipReady`.
5. **`duty_mesh`** flushes block + aggregation via `flush_all_pending_gossip` → `AggregationPublished`.
6. **`gossip_pool`** ingest accepts `SignedAggregatedAttestation` on `/aggregation/` (keeps attestation binding).

## Events

- `AggregationGossipReady { topic, data_root, payload_len, proof_len }`
- `AggregationPublished { topic, data_root, payload_len }`

## Still open (next)

- **D3**: proposer builds Type-2 from Type-1 cache for D5 block production.
- Production leanSig / leanVM (B1–B3) — mesh verify still fail-closed without backends.
- Operator fork-digest + bootnodes (A2/A3).

## Tests

- `cargo test -p ethean-node --lib aggregation_gossip --features libp2p-quic`
- `cargo test -p ethean-node --lib swarm_pump --features libp2p-quic`
- `cargo test -p ethean-node --lib gossip_pool`
