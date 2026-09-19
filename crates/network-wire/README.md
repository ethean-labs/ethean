# ethean-network-wire

Wire topics, message IDs, raw/framed Snappy, Status, and req/resp request shapes for Lean Consensus.

- Topics: `/leanconsensus/{fork}/…/ssz_snappy` (never `12345678`)
- Gossip: raw Snappy; req/resp: framed Snappy
- Interim `fork_identifier_bytes` / `fork_segment_hex` (SHA-256 prefix) until Phase 00 OSD pins the digest
- Message-ID preimage order remains an open Phase 00 gate; interim binding is documented in `message_id.rs`
