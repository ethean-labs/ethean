# Working-client plan: pq-devnet-5 target (2026-09-19)

## Verdict

pq-devnet-5 is **in progress** on [leanroadmap.org](https://leanroadmap.org/) (not a finished public mesh with published bootnodes). Ethean defaults `start` to that **network label** and dials only operator-supplied QUIC multiaddrs.

## Peer baseline

| Name | Use |
| --- | --- |
| Zeam | Primary Zig reference (devnet5 release shape, static nodes/ENR) |
| Ream | Primary Rust reference (`devnet5` feature, `--bootnodes`) |
| Peam | Secondary; older aggregation surfaces — not pin authority |
| Beam | Historical Lean name only |

## Sprint landings

1. CLI `--network pq-devnet-5` (default) + `--bootnodes` + `ETHEAN_BOOTNODES` + `config/networks/pq-devnet-5.bootnodes`
2. Dial bootnodes after QuicSwarm bind (feature `libp2p-quic`)
3. Root README Quick Start rewritten for real `ethean` binary
4. Clear offline warning when bootnode list is empty

## Follow-ups

- ~~Pin genesis fork digest to leanSpec (replace interim SHA-256 fork segment)~~ — **override path landed** (`--fork-digest` / env / file); still need the operator/leanSpec pin value for a live run.
- leanSig + leanVM production gates
- Status / blocks-by-root **stream** exchange against dialed peers (local Status + observe helpers ready)
- Goldfish only when leanSpec for the run requires it

## How to run (summary)

See root README Quick Start and [deployment.md](./deployment.md).
