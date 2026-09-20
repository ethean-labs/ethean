# Hive config.yaml / validators.yaml consume (2026-09-20)

## What landed

| Piece | Change |
| --- | --- |
| `ethean-genesis` | `parse_lean_network_config` / `genesis_from_lean_config` (Hive + quickstart YAML) |
| `ethean-node` | `validator_registry` parser for node id → indices |
| CLI | `--lean-config`, `--validator-registry`, `--node-id` |
| Hive entrypoint | Maps `HIVE_LEAN_NETWORK_CONFIG` / `HIVE_LEAN_VALIDATOR_REGISTRY_PATH` / `HIVE_NODE_ID` |

When `--lean-config` is set, genesis comes from `GENESIS_TIME` + `GENESIS_VALIDATORS`
(dual-key `attestation_*` / `proposal_*` field names accepted). Ephemeral smoke registry
is skipped.

## Example

```bash
ethean start --until-signal --ephemeral \
  --lean-config /tmp/ethean-runtime/config.yaml \
  --validator-registry /tmp/ethean-runtime/validators.yaml \
  --node-id ethean_0
```

## Still open

- Wire attestation registry key into attester duties — **landed**
  (see [`local-attester-duty-2026-09-20.md`](./local-attester-duty-2026-09-20.md))
- Upstream ethereum/hive `clients/ethean` (+ add `ethean` to prepare_lean_client_assets.py)
- Apply `ATTESTATION_COMMITTEE_COUNT` into the live profile when Hive sets it
- Propose only when slot proposer matches an owned registry index
