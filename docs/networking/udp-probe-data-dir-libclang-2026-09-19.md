# UDP Status probe, data-dir, and libclang helper (2026-09-19)

## Networking

- `parse_quic_udp` / `QuicUdpAddr` for `/ip4/.../udp/.../quic-v1`
- `probe_udp_status`: UDP Status path probe with optional reply (not a TLS/QUIC crypto handshake)
- `dial_quic` / `dial_quic_pending`: still fail-closed pending libp2p QUIC-v1 swarm
- `SwarmFacade::probe_peer_status` wraps the UDP probe

## Storage / CLI

- `EtheanClient::open_data_dir` / `with_genesis_store`
- `ethean start --data-dir PATH` (needs `ethean-storage` built with `rocksdb`)
- `tools/release/check-libclang.ps1` locates `libclang.dll` for RocksDB builds

## Validator

- `ethean validator` prints `FfiStatus` gaps and stays fail-closed without backends

## Verification

```text
cargo test -p ethean-network -p ethean-node --lib
cargo check -p ethean
```

network 15 tests, node 19 tests.

## Still external

libp2p QUIC swarm crypto dial; LLVM/libclang for RocksDB on this host; leanSig/leanVM link pins.
