# Lean wire conformance (leanSpec lstar) — 2026-09-24

The pq-devnet-5 retrospective names spec drift as the main interop failure, so the
wire layer was re-checked against leanSpec `0b7d33ec` (fork `lstar`) and ream.
Every item below is locked by unit tests in `crates/network-wire` or `crates/network`.

| Item | Before | Now |
| --- | --- | --- |
| Gossip fork segment | `sha256(fork_name)[..4]`, `12345678` rejected | lstar `GOSSIP_DIGEST = "12345678"`; operator override accepts `12345678` and `0x12345678` |
| Gossipsub identity | signed messages, permissive validation | `MessageAuthenticity::Anonymous`, `ValidationMode::Anonymous` |
| Message id | private 32-byte hash, local dedup only | `sha256(domain ‖ u64le(len(topic)) ‖ topic ‖ data)[:20]`, domain `01000000` for valid snappy (decompressed data), `00000000` otherwise |
| Mesh parameters | libp2p defaults | heartbeat 700 ms, D 8 / D_low 6 / D_high 12, D_lazy 6, mcache 6/3, seen TTL 24 s |
| Status | genesis root + fork string + checkpoints | SSZ `Status{finalized: Checkpoint, head: Checkpoint}`, 80 bytes |
| Req/resp request | `u32 LE compressed_len ‖ raw snappy` | `varint(uncompressed_len) ‖ snappy framed stream` |
| Req/resp response | one custom frame with an inline block list | per chunk `code ‖ varint(uncompressed_len) ‖ snappy frame`, one `SignedBlock` per chunk, read to EOF |
| Response codes | 1/2/3 renumbered | SUCCESS 0, INVALID_REQUEST 1, SERVER_ERROR 2, RESOURCE_UNAVAILABLE 3 |
| BlocksByRoot request | `u32 count ‖ roots` | SSZ `List[Bytes32, 1024]` (offset 4 then roots) |
| BlocksByRange request | 24 bytes with `step` | 16 bytes `start_slot ‖ count` |
| Attestation subnets | 4 | follows `ATTESTATION_COMMITTEE_COUNT` (lstar 1) |
| Size limits | 2 MiB | `MAX_PAYLOAD_SIZE` 10 MiB, message size `32 + n + n/6 + 1024` like ream |

Chunk boundaries are found by the declared uncompressed length: the decoder runs a
snappy frame decoder until exactly that many bytes came out and reports how many
wire bytes it consumed (`snappy::decompress_frame_prefix`). Walking snappy chunk
headers is wrong because the next response chunk starts with a code byte, not a
stream identifier. An empty payload is the bare stream identifier
(`ff 06 00 00 sNaPpY`). The former snappy expansion-ratio guard was removed: the
spec bounds only the uncompressed size.

Open question for the interop channel: leanSpec builds topics with `12345678`
while ream's topic tests use `0x12345678`. Ethean follows leanSpec and normalises
either form on input.

Vectors: `leanSpec/tests/consensus/lstar/networking/` (gossip message id, topic and
fork, req/resp codec and response stream, snappy block and frame, varint, ENR).
