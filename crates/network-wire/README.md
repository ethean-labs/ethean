# ethean-network-wire

Wire topics, message IDs, raw/framed Snappy, Status, and req/resp request shapes for Lean Consensus.

- Topics: `/leanconsensus/{fork}/…/ssz_snappy` (never `12345678`)
- Gossip: raw Snappy; req/resp: framed Snappy
- `fork_identifier_bytes` and message-ID preimage order remain Phase 00 open gates; interim mappings are documented in code
