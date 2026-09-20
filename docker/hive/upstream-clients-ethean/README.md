# Upstream ethereum/hive `clients/ethean` drop-in

Copy this folder to `ethereum/hive/clients/ethean/` when opening the registration PR.

## Prerequisites

1. Build the local image from the Ethean repo root:

```bash
docker build -f docker/hive/Dockerfile -t ethpandaops/ethean:local .
```

2. Hive Lean simulator must prepare `config.yaml` + `validators.yaml` under
   `/tmp/ethean-runtime/` (or set `HIVE_LEAN_NETWORK_CONFIG` /
   `HIVE_LEAN_VALIDATOR_REGISTRY_PATH`). This entrypoint **requires** both files
   (same contract as Ream).

## Hive registration checklist

1. Copy this directory → `clients/ethean/` in ethereum/hive.
2. Append to `simulators/lean/clients/devnet5.yaml` (and optionally `devnet4.yaml`):

```yaml
- client: ethean
  nametag: devnet5
```

3. Add a matrix line to `simulators/lean/config/lean-devnets.txt`:

```text
ethean=devnet4,devnet5
```

4. If the Lean asset preparer lists client ids, add `ethean` there so Hive
   injects prepared genesis/registry under the expected paths.

5. Smoke:

```bash
./hive --sim lean \
  --client-file simulators/lean/clients/devnet5.yaml \
  --client ethean \
  --docker.output
```

## Build args

| Arg | Default | Purpose |
| --- | --- | --- |
| `baseimage` | `ethpandaops/ethean` | Pre-built client image |
| `tag` | `local` | Image tag |
| `runtime_baseimage` | `debian:bookworm-slim` | Thin runtime layer |

## Honesty

- Lean HTTP is `/lean/v1/…` only (no Beacon `/eth/v1`).
- Proposal XMSS needs a `leansig-backend` image build; attester registry keys load
  when present.
- `ATTESTATION_COMMITTEE_COUNT` is taken from prepared `config.yaml` (profile +
  gossip subnets), not a separate Hive CLI flag.
