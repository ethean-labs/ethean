# pq-devnet-5 client run verification (2026-09-19)

## Verdict

- **Public pq-devnet-5 mesh:** not joinable yet. `config/networks/pq-devnet-5.bootnodes`
  is empty; leanroadmap / lean-quickstart expect operator `nodes.yaml` ENRs for each
  interop run. No permanent public QUIC multiaddrs were found.
- **Client binary:** works under the `pq-devnet-5` label offline and over a **local
  two-process QUIC mesh** with Status request_response.

## Runs performed

### 1. Offline smoke

```text
ethean start --network pq-devnet-5 --ticks 3
```

Observed: QuicSwarm bind, fork segment `aa4c6403`, offline bootnode warning, duty
loop completed (`ticks_accepted=3`). leanSig/leanVM gates remain fail-closed
(`ready=false`).

### 2. Integration test

```text
cargo test -p ethean-network --features libp2p-quic --test status_mesh
```

`two_nodes_dial_and_status_response` passed (dial + StatusResponse round-trip).

### 3. Two-process binary mesh

1. Peer A: `ethean start --network pq-devnet-5 --until-signal`
2. Peer B: dial A's `dialable` multiaddr rewritten to `127.0.0.1`

Observed on both peers:

- bootnode dial succeeded
- Status handshake queued and flushed on `/leanconsensus/req/status/1/ssz_snappy`
- `Status handshake completed; heads match`

## Fixes needed for this verification

Duty loops previously did not poll QuicSwarm after boot, so a listening
`--until-signal` node could not accept late dials. Wall / until-signal now call
`apply_network_budget` each tick (`duty_mesh` + `duty_network`).

## How to re-check locally

```bash
cargo build -p ethean --release
RUST_LOG=info ./target/release/ethean start --network pq-devnet-5 --ticks 3
cargo test -p ethean-network --features libp2p-quic --test status_mesh
```

For two binaries: start A with `--until-signal`, copy the logged `dialable=…`
(use `127.0.0.1` on the same host), then:

```bash
./target/release/ethean start --network pq-devnet-5 --ticks 3 --wall-clock \
  --bootnodes '/ip4/127.0.0.1/udp/<port>/quic-v1/p2p/<peer-id>'
```

Live interop still needs operator bootnodes + matching fork digest in
`pq-devnet-5.bootnodes` / `pq-devnet-5.forkdigest`.
