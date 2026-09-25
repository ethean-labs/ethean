# Phase 10 — QUIC, Gossip, and Request/Response

## Pinned inputs

- Snapshot `LC-D5-2026-09-19`; Ethean planning baseline `880982f`; `leanSpec` `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54`; Phase 00 fixture SHA-256 manifest.
- Wire topics: `/leanconsensus/{fork}/block/ssz_snappy`, `/leanconsensus/{fork}/attestation_<subnet>/ssz_snappy`, and `/leanconsensus/{fork}/aggregation/ssz_snappy`; `{fork}` is profile-derived and never the dummy `12345678`.
- RPC IDs: `/leanconsensus/req/status/1/ssz_snappy`, `/leanconsensus/req/blocks_by_root/1/ssz_snappy`, and `/leanconsensus/req/blocks_by_range/1/ssz_snappy`; request maximum is 1,024 blocks.
- Transport is libp2p QUIC-v1 over UDP with TLS identity bound to PeerId. Payloads are canonical SSZ plus the exact raw-Snappy (gossip) and framed-Snappy (req/resp) modes from the pinned spec.
- Rust `1.97.1`; Cargo lockfile, libp2p/QUIC/Snappy versions, build image, topic limits, stream limits, peer/IP admission caps, and timeout profile are immutable within the phase.
- Differential peers: Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, gean `b78f6d737f4df57a72d5e230635681235fda8024`, and Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.
- Evidence only (not authority): Zeam pinned snapshot documents raw-vs-framed Snappy mismatches and decompression-limit edge cases in gossip/reqresp paths; Peam pinned snapshot documents topic template and fork-identifier patterns under `/leanconsensus/{fork}/…/ssz_snappy`. Ethean follows leanSpec and the Phase 00 compatibility ledger; peer notes inform differential tests and abuse fixtures only.

## Objective

Deliver interoperable QUIC transport, SSZ+Snappy codecs, profile-derived topic and message-ID handling, bounded Gossipsub validation, peer admission limits, Status handshake, and blocks-by-root/range retrieval without allowing malformed or expensive input to reach consensus unchecked.

## Non-goals

- No TCP/WebSocket compatibility mode, `/eth2/` topics, mDNS production discovery, set reconciliation, candidate-body topic, or unpinned Gossipsub v2 claim.
- Discovery beyond static authenticated ENRs is added only if the pinned profile requires it.
- Networking never decides consensus validity or stores unverified pending blocks as serveable data.
- Peer Snappy or topic quirks are not copied unless the pinned spec and fixtures require them.

## Entry criteria

- Phases 03–09 pass; canonical wire types, message roots, validation outcomes, node commands, and sync interface are stable.
- Topic strings, message-ID vectors, Snappy mode, fork identity, RPC framing, response codes, and stream termination are present in the compatibility ledger.
- Per-message bytes, decompressed bytes, allocation, verification, queue, peer, IP, connection, stream, and request budgets are approved.

## Exact old and new paths

Replace then delete all code in `src/network/`, networking glue in `src/integration/network_storage.rs` and `src/integration/sync_coordinator.rs`, and legacy dependencies/features in `Cargo.toml`.

Create:

- `crates/ethean-network-wire/src/{lib.rs,topics.rs,message_id.rs,snappy.rs,status.rs,reqresp.rs,limits.rs}`.
- `crates/ethean-network/src/{lib.rs,transport.rs,identity.rs,swarm.rs,peer_manager.rs,admission.rs}`.
- `crates/ethean-network/src/gossip/{mod.rs,codec.rs,validation.rs,scoring.rs}`.
- `crates/ethean-network/src/reqresp/{mod.rs,codec.rs,handler.rs,tracker.rs}`.
- `tests/fixtures/network/`, `tests/interop/network.rs`, `tests/negative/network_abuse.rs`, and `tests/soak/quic.rs`.

Every new code directory gets an English `README.md`; every hand-written source file is at most 2000 lines.

## Ordered tasks

1. Implement profile-derived fork/topic/RPC identifiers and exact message-ID calculation; fixture-test valid and invalid Snappy domains and byte order.
2. Encode canonical SSZ then raw Snappy for gossip and framed Snappy for req/resp; before decompression enforce compressed size, declared output, ratio, elapsed time, and allocation limits. Reject trailing bytes and noncanonical SSZ.
3. Build QUIC-v1/TLS transport with persistent node identity, PeerId binding, safe certificates, UDP buffer guidance, idle/handshake timeout, stream caps, reconnect backoff, IPv4/IPv6, and graceful close.
4. Validate ENR identity/signature, fork/profile, address, sequence, self/duplicate status, subnet metadata, and configured trust. Static peers use the same admission limits.
5. Configure anonymous Gossipsub, mesh/fanout/heartbeat/duplicate-cache limits, application validation feedback, score decay, per-topic caps, and peer/IP rate limits from the pinned profile.
6. Decode to a bounded wire object, then perform topic/type/subnet, temporal, topology, duplicate, signature/proof, and parent checks. Map each result explicitly to ACCEPT, IGNORE, or REJECT.
7. Queue unknown-parent objects in bounded TTL structures and request ancestry; never score honest missing-parent behavior as cryptographic invalidity.
8. Exchange Status at connection, rejecting network/genesis/profile mismatch. Track head/finalized consistency and feed peer horizons to sync without trusting claims.
9. Implement blocks-by-root with request-order handling, missing roots, response codes, chunk limits, timeout, cancellation, root verification, and per-peer concurrency.
10. Implement blocks-by-range with exact start/count/step semantics, skipped slots, ordered unique responses, canonical membership checks, partial-stream handling, and retry/bad-peer rotation.
11. Bound request IDs, pending maps, unknown-parent graphs, served bytes, slow readers/writers, and negative caches; prune on disconnect and shutdown.
12. Add progress health for swarm event loop, mesh, RPC completion, stream age, and last valid message so a live process with a wedged network is unhealthy.

## Deletion obligations

- Delete simulated connections, in-memory gossip IDs, mock block bytes, TCP/WebSocket security transport, Beacon topics/bootnodes, and empty `NetworkManager`.
- Remove legacy `noise`, `yamux`, TCP, DNS, and mDNS features unless an explicit pinned requirement remains.
- No old/new transport switch or legacy decoder remains in a production build.
- Repository scans may mention deleted paths only in migration/deletion records.

## Security/spec risks

- Message-ID byte-order disagreement can partition duplicate suppression; the compatibility ledger records peer divergence for differential tests only.
- Snappy bombs, malformed offsets, proof payloads, slow streams, and maximum range requests can exhaust resources; Zeam evidence highlights framing-mode confusion as a test vector, not a design choice.
- Static ENRs are bootstrapping, not resilient discovery; topology assumptions must be explicit.
- Gossipsub without scoring/application feedback enables invalid-message amplification.
- Stream-table age can cause long-run CPU collapse; profile live and historical streams.
- Empty-body Status compatibility is accepted only if its exact pinned vector says so.

## Positive and negative fixtures

- Positive: all topics, message IDs, SSZ/Snappy payloads, Status, roots/range requests, response codes, framing, chunking, maximum boundaries, and Peam-style fork/topic strings derived from the pinned profile.
- Negative: malformed/truncated/trailing SSZ, Snappy bomb, wrong topic/subnet/fork, invalid message ID, stale/future objects, oversized streams, slowloris, duplicate flood, ENR poisoning, network mismatch, and Zeam-documented raw/framed Snappy confusion cases.
- Fixture provenance and hashes live in `tests/fixtures/network/manifest.toml` and the Phase 00 SHA-256 manifest.

## Interop and differential tests

- Every Ethean/peer ordered pair connects over QUIC, exchanges Status and three gossip families, retrieves missing ancestors by root/range, and rejects mixed profiles clearly.
- Differential matrix records message-ID preimage order, Snappy mode, and topic fork encoding against Ream, Zeam, Lantern, gean, and Peam snapshots; disagreement is reported, never majority-voted.
- Recovery: disconnect during each frame, restart with pending requests, peer rotation, QUIC handshake failure, UDP loss, stream reset, partition/heal, and swarm-worker replacement.
- Soak: 72-hour connection churn and stream reuse with CPU, memory, FD, task, and table-size leak checks.

## Validation commands

```text
cargo +1.97.1 test -p ethean-network-wire --locked
cargo +1.97.1 test -p ethean-network --locked
cargo +1.97.1 test --test network --release --locked --features interop
cargo +1.97.1 test --test network_abuse --release --locked
cargo +1.97.1 test --test quic --release --locked -- --ignored
cargo +1.97.1 clippy -p ethean-network-wire -p ethean-network --all-targets --locked -- -D warnings
rg -n "/eth2/|NetworkManager|mock.*gossip|yamux|noise" crates src Cargo.toml
```

## Exit criteria

- Every pinned codec/message-ID fixture passes with zero unsupported cases.
- Mixed-client matrix passes QUIC, Status, gossip, root/range sync, malformed-frame rejection, and profile mismatch.
- Resource-abuse tests remain within approved CPU/RAM/FD/bandwidth bounds and preserve chain-owner progress.
- The 72-hour soak has no stream-table growth, wedge, or reconnect storm.
- Mock gossip, TCP/WS Beacon topics, empty `NetworkManager`, and legacy networking are deleted; touched source files remain within 2000 lines.

## Rollback/data policy

Peer scores, pending requests, and connection state are disposable. Node identity is durable and must not be regenerated silently. A network rollback is allowed only when protocol/profile IDs and persisted chain fingerprint match; otherwise startup fails. Unverified network objects are never promoted to durable canonical storage.

## Artifacts/evidence

- Codec vectors, packet captures without secrets, interop matrix, peer-score scenarios, abuse budgets, QUIC/stream profiles, 72-hour leak report, profile-mismatch logs, and Zeam/Peam differential notes in `artifacts/phase-10/`.

## Dependencies

Depends on Phases 03–09. Phase 11 consumes the retrieval interface; Phase 12 scrapes fleet/network health; Phase 13 adds chaos and release qualification.
