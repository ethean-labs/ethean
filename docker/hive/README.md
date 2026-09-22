# Ethean Hive client scaffold

Local Docker assets for registering Ethean in [ethereum/hive](https://github.com/ethereum/hive)
`simulators/lean`.

## Build

From the repository root:

```bash
docker build -f docker/hive/Dockerfile -t ethpandaops/ethean:local .
```

The image contains `ethean` and `ethean-prover` (leanMultisig prover used by
aggregators and proposers). XMSS is native; no build features are needed.

## Entrypoint env (Lean simulator)

| Variable | Mapping |
| --- | --- |
| `HIVE_LEAN_DEVNET_LABEL` | `devnet5` → `--network pq-devnet-5` |
| `HIVE_BOOTNODES` | `--bootnodes` (omit when empty/`none`) |
| `HIVE_FORK_DIGEST` / `HIVE_LEAN_FORK_DIGEST` | `--fork-digest` |
| `HIVE_VALIDATORS` | `--validators` |
| `HIVE_IS_AGGREGATOR` | `0` → `--no-aggregator` |
| `HIVE_METRICS_ENABLED` | `1` keeps scrape on `:9100` |
| `HIVE_HTTP_ENABLED` | `0` → `--no-http` (default on `:5052`) |
| `HIVE_HTTP_ADMIN_TOKEN` | `--http-admin-token` for admin routes |
| `HIVE_LISTEN_PORT` | `--listen-port` (default `9000`) |
| `HIVE_LEAN_NETWORK_CONFIG` | `--lean-config` (`config.yaml`) |
| `HIVE_LEAN_VALIDATOR_REGISTRY_PATH` | `--validator-registry` |
| `HIVE_NODE_ID` | `--node-id` (default `ethean_0`) |
| `HIVE_ETHEAN_PROVER_BIN` | exports `ETHEAN_PROVER_BIN` (defaults to the bundled `ethean-prover`) |

Operator plug-in order (fork-digest, bootnodes, leanVM, leansig, Hive):
[`docs/pq-devnet-operator-plug-in-checklist-2026-09-20.md`](../../docs/pq-devnet/pq-devnet-operator-plug-in-checklist-2026-09-20.md).

Always runs `ethean start --until-signal --ephemeral` with Lean HTTP on `0.0.0.0:5052`
and QUIC on UDP `:9000` (override via `HIVE_LISTEN_PORT`). When Hive injects prepared
assets, genesis comes from `config.yaml` instead of the ephemeral smoke registry.

## Upstream registration (not in this repo)

Drop-in files live in [`upstream-clients-ethean/`](./upstream-clients-ethean/) —
copy that folder to ethereum/hive `clients/ethean/`, append
[`client-devnet5.yaml`](./client-devnet5.yaml) into `simulators/lean/clients/devnet5.yaml`,
and add the lean-devnets snippet. Details:
[`docs/hive-upstream-clients-ethean-dropin-2026-09-20.md`](../../docs/hive-testing/hive-upstream-clients-ethean-dropin-2026-09-20.md).

## Known gaps vs Ream Hive client

- Lean HTTP serves `/lean/v1/…` only (no Beacon `/eth/v1`).
- Attestation checks use the head state registry; the spec's target-state lookup needs a fork-choice store.
- Upstream ethereum/hive merge still required (drop-in is ready to copy).
- `ATTESTATION_COMMITTEE_COUNT` from prepared `config.yaml` lands on the profile and
  gossip subnet count (see
  [`docs/attestation-committee-count-profile-subnets-2026-09-20.md`](../../docs/networking/attestation-committee-count-profile-subnets-2026-09-20.md)).
