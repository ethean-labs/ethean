#!/usr/bin/env bash
# ethereum/hive clients/ethean entrypoint (strict Lean asset paths).
# Copy this directory to ethereum/hive/clients/ethean/ when opening the upstream PR.
set -euo pipefail

DEVNET_LABEL="${HIVE_LEAN_DEVNET_LABEL:-devnet5}"
NODE_ID="${HIVE_NODE_ID:-ethean_0}"
BOOTNODES="${HIVE_BOOTNODES:-}"
FORK_DIGEST="${HIVE_FORK_DIGEST:-${HIVE_LEAN_FORK_DIGEST:-}}"
ASSET_ROOT="/tmp/ethean-runtime"
NETWORK_CONFIG="${HIVE_LEAN_NETWORK_CONFIG:-$ASSET_ROOT/config.yaml}"
VALIDATOR_REGISTRY="${HIVE_LEAN_VALIDATOR_REGISTRY_PATH:-$ASSET_ROOT/validators.yaml}"
ETHEAN_BIN="${ETHEAN_BIN:-/usr/local/bin/ethean}"

case "$DEVNET_LABEL" in
  devnet5|pq-devnet-5)
    NETWORK="pq-devnet-5"
    ;;
  devnet4|pq-devnet-4)
    NETWORK="pq-devnet-4"
    ;;
  local|smoke)
    NETWORK="local"
    ;;
  *)
    echo "Unsupported HIVE_LEAN_DEVNET_LABEL=$DEVNET_LABEL (use devnet5|devnet4|local)" >&2
    exit 1
    ;;
esac

if [ ! -f "$NETWORK_CONFIG" ]; then
  echo "Missing prepared Lean network config at $NETWORK_CONFIG" >&2
  exit 1
fi

if [ ! -f "$VALIDATOR_REGISTRY" ]; then
  echo "Missing prepared Lean validator registry at $VALIDATOR_REGISTRY" >&2
  exit 1
fi

FLAGS=(
  start
  --until-signal
  --ephemeral
  --no-banner
  --network "$NETWORK"
  --metrics-address 0.0.0.0
  --metrics-port 9100
  --http-address 0.0.0.0
  --http-port 5052
  --listen-port "${HIVE_LISTEN_PORT:-9000}"
  --lean-config "$NETWORK_CONFIG"
  --validator-registry "$VALIDATOR_REGISTRY"
  --node-id "$NODE_ID"
)

if [ -n "$BOOTNODES" ] && [ "$BOOTNODES" != "none" ]; then
  FLAGS+=(--bootnodes "$BOOTNODES")
fi

if [ -n "$FORK_DIGEST" ]; then
  FLAGS+=(--fork-digest "$FORK_DIGEST")
fi

if [ -n "${HIVE_VALIDATORS:-}" ]; then
  FLAGS+=(--validators "$HIVE_VALIDATORS")
fi

if [ "${HIVE_IS_AGGREGATOR:-1}" = "0" ]; then
  FLAGS+=(--no-aggregator)
fi

if [ "${HIVE_METRICS_ENABLED:-0}" != "1" ]; then
  FLAGS+=(--no-metrics)
fi

if [ "${HIVE_HTTP_ENABLED:-1}" = "0" ]; then
  FLAGS+=(--no-http)
fi

if [ -n "${HIVE_HTTP_ADMIN_TOKEN:-}" ]; then
  FLAGS+=(--http-admin-token "$HIVE_HTTP_ADMIN_TOKEN")
fi

export RUST_LOG="${RUST_LOG:-info}"

exec "$ETHEAN_BIN" "${FLAGS[@]}"
