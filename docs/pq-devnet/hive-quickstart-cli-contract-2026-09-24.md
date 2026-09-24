# Hive / lean-quickstart CLI contract (2026-09-24)

Ethean `start` flags now accept the same names Hive and lean-quickstart pass to
Ream. Legacy Ethean names remain as aliases. This note is the mapping, not a
copy of peer crate layout.

## Flag mapping

| Hive / lean-quickstart input | Ethean flag | Notes |
| --- | --- | --- |
| `--network <label>` | `--network <label\|path>` (default `pq-devnet-4`) | A file path loads that Lean `config.yaml` under the `local` profile |
| `--network` path to `config.yaml` | `--network <path>` or `--lean-config <path>` | `--lean-config` wins when both name a file |
| `--bootnodes none` | `--bootnodes none` | Empty dial list; skips `ETHEAN_BOOTNODES` and `config/networks/<label>.bootnodes` |
| `--bootnodes` CSV of QUIC multiaddrs | `--bootnodes <csv>` | Same as before |
| `--bootnodes` CSV of `enr:…` | `--bootnodes <csv>` | Decoded to `/ip4/<ip>/udp/<quic\|udp>/quic-v1/p2p/<peerid>` |
| `nodes.yaml` (YAML list of ENR / multiaddr) | `--bootnodes <path>` | lean-quickstart genesis file shape |
| `--private-key-path` / Hive `node.key` | `--node-key` (alias `--private-key-path`) | 64 hex chars, optional `0x`, trailing newline tolerated |
| `--socket-address` | `--socket-address` (default `0.0.0.0`) | QUIC listen interface |
| `--socket-port` | `--listen-port` (alias `--socket-port`, default `9000`) | `0` = OS ephemeral |
| `--validator-registry-path` | `--validator-registry` (alias `--validator-registry-path`) | plus `--node-id` (default `ethean_0`) |
| `--is-aggregator` | `--is-aggregator` | With a genesis file, aggregator is **off** unless this flag is set; local finality is off |
| `--no-aggregator` | `--no-aggregator` | Still wins over `--is-aggregator` |
| `--aggregate-subnet-ids` | `--aggregate-subnet-ids` | Recorded; Ethean already subscribes to every attestation subnet |
| `--attestation-committee-count` | `--attestation-committee-count` | Overrides `ATTESTATION_COMMITTEE_COUNT` from `config.yaml` |
| `--checkpoint-sync-url` | `--checkpoint-sync-url` | Accepted; logs a warning until empty-datadir SSZ fetch is wired |
| `--metrics` | `--metrics` | No-op keep-on for scrape HTTP (already default). Does **not** start Docker |
| Grafana / Prometheus compose | `--observability-stack` | `docker compose up` in `deploy/observability` |
| HTTP bind | `--http-address` / `--http-port` (defaults `127.0.0.1:5052`) | Env fallback `ETHEAN_HTTP_ADDRESS` |
| Metrics bind | `--metrics-address` / `--metrics-port` (defaults `127.0.0.1:9100`) | Env fallback `ETHEAN_METRICS_ADDRESS` |

Solo smoke (no genesis file) keeps the historical defaults: aggregator on,
local finality on, unless `--no-aggregator` / `--no-local-finality`.

## Node key

The QUIC swarm identity is a **secp256k1** libp2p key (Hive / quickstart
derive `/ip4/<ip>/udp/9000/quic-v1/p2p/<peerid>` from it).

1. `--node-key <file>` (alias `--private-key-path`): load 32-byte secret as 64 hex chars.
2. Else if `--data-dir` is set and `--ephemeral` is not: persist / reuse `<data-dir>/node.key`.
3. Else: generate a per-run key (ephemeral).

Boot logs the PeerId and the dialable multiaddr (`<listen>/p2p/<peerid>`).
`/lean/v1/node/identity` uses that PeerId.

## ENR bootnodes

`enr:` records are EIP-778 RLP (`signature`, `seq`, then `id`/`secp256k1`/`ip`/`udp`/`quic`/`quic6`/`ip6`).
Ethean does not verify the discv5 signature; QUIC authenticates the PeerId in
the multiaddr. IPv4 uses `quic` then `udp`; IPv6 uses `quic6` then `udp6`.

Vector from lean-quickstart README `nodes.yaml`:

```
enr:-IW4QMn2QUYENcnsEpITZLph3YZee8Y3B92INUje_riQUOFQQ5Zm5kASi7E_IuQoGCWgcmCYrH920Q52kH7tQcWcPhEBgmlkgnY0gmlwhH8AAAGEcXVpY4IjKIlzZWNwMjU2azGhAhMMnGF1rmIPQ9tWgqfkNmvsG-aIyc9EJU5JFo3Tegys
```

decodes to `/ip4/127.0.0.1/udp/9000/quic-v1/p2p/16Uiu2HAkvi2sxT75Bpq1c7yV2FjnSQJJ432d6jeshbmfdJss1i6f`.
