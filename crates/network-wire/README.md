# ethean-network-wire

Wire topics, message IDs, raw/framed Snappy, Status, and req/resp request shapes for Lean Consensus.

- Topics: `/leanconsensus/{fork}/…/ssz_snappy`. The lstar default fork segment is `GOSSIP_DIGEST` (`12345678`), with no `0x` prefix (leanSpec). Operator `--fork-digest` / `*.forkdigest` may include `0x`; the prefix is stripped before the topic is built. Ream currently embeds `0x12345678` in its own tests — that is a known discrepancy; Ethean follows leanSpec.
- Gossip: raw Snappy; message-id is `SHA256(domain ‖ uint64_le(len(topic)) ‖ topic ‖ data)[:20]` with domain `0x01000000` (valid snappy, decompressed data) or `0x00000000` (otherwise).
- Req/resp: unsigned LEB128 uncompressed length + Snappy framed SSZ. Responses are `code(1) ‖ varint ‖ snappy frame`, several chunks per stream for blocks.
- Status is SSZ `Status{finalized: Checkpoint, head: Checkpoint}` (80 bytes).
- Response codes: SUCCESS=0, INVALID_REQUEST=1, SERVER_ERROR=2, RESOURCE_UNAVAILABLE=3.
