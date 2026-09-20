# Fork digest override and local Status (2026-09-19)

## Landed

- `fork_segment_resolve(fork_name, override)` — operator 8-hex digest wins over interim SHA-256(`lstar`) hash.
- CLI `--fork-digest`, env `ETHEAN_FORK_DIGEST`, file `config/networks/pq-devnet-5.forkdigest`.
- QuicSwarm binds topics as `/leanconsensus/{segment}/…` using the resolved segment.
- `local_status` builds Lean Status (genesis/head/finalized) for future peer handshake; logs on start.
- `observe_remote_status` updates sync lag when a remote Status is available.

## Verified

```text
cargo run -p ethean -- start --ticks 1 --fork-digest aabbccdd
# topic=/leanconsensus/aabbccdd/block/ssz_snappy
```

## Still open

- Wire Status over `/leanconsensus/req/status/1/ssz_snappy` streams after dial (QuicSwarm req/resp).
- Replace scaffolding Status codec with leanSpec SSZ Status when the pin is fixed.
- Real operator digest + bootnodes for a live pq-devnet-5 run.
