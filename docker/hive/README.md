# Ethean Hive client scaffold

Local Docker assets for registering Ethean in [ethereum/hive](https://github.com/ethereum/hive)
`simulators/lean`.

Published image: `ghcr.io/ethean-labs/ethean` (see
[`docs/release/docker-images.md`](../../docs/release/docker-images.md)). The
GitHub owner of the repository that runs CI owns the GHCR namespace.

## Build

From the repository root:

```bash
# Root image (CI recipe)
docker build -t ghcr.io/ethean-labs/ethean:local .

# Hive-oriented local image (this Dockerfile + docker/hive/ethean.sh)
docker build -f docker/hive/Dockerfile -t ghcr.io/ethean-labs/ethean:local .
```

The image contains `ethean` and `ethean-prover` (leanMultisig prover used by
aggregators and proposers). XMSS is native; no build features are needed.

## Local Hive entrypoint (`ethean.sh`)

`docker/hive/ethean.sh` is the flexible smoke entrypoint (network labels,
optional prepared assets). Hive simulator runs use the drop-in below.

## Upstream registration

Drop-in files live in [`upstream-clients-ethean/`](./upstream-clients-ethean/) —
copy that folder to ethereum/hive `clients/ethean/` (already applied in the
workspace `./hive` clone). Structure matches `hive/clients/ream/`:

| File | Role |
| --- | --- |
| `Dockerfile` | Thin wrapper of `ghcr.io/ethean-labs/ethean:devnet5` |
| `Dockerfile.git` | Build from source (`ethean-labs/ethean`) |
| `ethean.sh` | Strict Hive entrypoint (requires prepared assets) |
| `hive.yaml` | `roles: [lean]` |
| `validators.yaml` | `ethean_0` … `ethean_15` empty lists |

Simulator registration (sibling `./hive` clone):

- `simulators/lean/clients/devnet4.yaml` / `devnet5.yaml` — `client: ethean`
- `simulators/lean/config/lean-devnets.txt` — `ethean=devnet4,devnet5`
- `simulators/lean/src/utils/util.rs` — `lean_client_kind` list (multiaddr bootnodes, not ENR)
- `simulators/lean/helper/prepare_lean_client_assets.py` — ream-shaped writer

Hive smoke (from the hive repo root, Docker daemon required):

```bash
./hive --sim lean \
    --client-file simulators/lean/clients/devnet5.yaml \
    --client ethean \
    --docker.output \
    --results-root ./workspace/logs
```

PR checklist: [`docs/hive-testing/hive-quickstart-registration-2026-09-24.md`](../../docs/hive-testing/hive-quickstart-registration-2026-09-24.md).

## Drop-in `ethean.sh` env (Lean simulator)

| Variable | Mapping |
| --- | --- |
| `HIVE_LEAN_DEVNET_LABEL` | `devnet5` / `devnet4` (same binary; genesis from `config.yaml`) |
| `HIVE_BOOTNODES` | `--bootnodes` (pass-through, including `none`) |
| `HIVE_LEAN_FORK_DIGEST` | `--fork-digest` |
| `HIVE_CHECKPOINT_SYNC_URL` | `--checkpoint-sync-url` |
| `HIVE_IS_AGGREGATOR` | `1` → `--is-aggregator` |
| `HIVE_AGGREGATE_SUBNET_IDS` | `--aggregate-subnet-ids` when set |
| `HIVE_ATTESTATION_COMMITTEE_COUNT` | `--attestation-committee-count` when set and not `1` |
| `HIVE_CLIENT_PRIVATE_KEY` | `--node-key` tmpfile; else `$ASSET_ROOT/node.key` |
| `HIVE_METRICS_ENABLED` | `1` → `--metrics --metrics-address 0.0.0.0 --metrics-port 8080` |
| `HIVE_LEAN_NETWORK_CONFIG` | `--network` path (default `/tmp/ethean-runtime/config.yaml`) |
| `HIVE_LEAN_VALIDATOR_REGISTRY_PATH` | `--validator-registry-path` (default `/tmp/ethean-runtime/validators.yaml`) |
| `HIVE_NODE_ID` | `--node-id` (default `ethean_0`) |
| `HIVE_LEAN_TEST_DRIVER` | exported through when `1` |
| `RUST_LOG` | default `info` |

Always runs `ethean start --until-signal --ephemeral --no-banner` with Lean HTTP
on `0.0.0.0:5052` and QUIC on UDP `:9000`. Prepared assets are required.

## Known gaps vs Ream Hive client

- Lean HTTP serves `/lean/v1/…` only (no Beacon `/eth/v1`).
- Attestation checks use the head state registry; the spec's target-state lookup needs a fork-choice store.
- Upstream ethereum/hive merge still required (drop-in is ready to copy / already in the workspace clone).
