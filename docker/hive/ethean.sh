#!/usr/bin/env bash
# Hive entrypoint for Ethean Lean Consensus Client.
# Maps ethereum/hive Lean simulator env vars onto `ethean start`.
set -euo pipefail

DEVNET_LABEL="${HIVE_LEAN_DEVNET_LABEL:-devnet5}"
BOOTNODES="${HIVE_BOOTNODES:-}"
FORK_DIGEST="${HIVE_FORK_DIGEST:-${HIVE_LEAN_FORK_DIGEST:-}}"
NODE_ID="${HIVE_NODE_ID:-ethean_0}"
NETWORK_CONFIG="${HIVE_LEAN_NETWORK_CONFIG:-}"
VALIDATOR_REGISTRY="${HIVE_LEAN_VALIDATOR_REGISTRY_PATH:-}"
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

if [ -n "$NETWORK_CONFIG" ]; then
  if [ ! -f "$NETWORK_CONFIG" ]; then
    echo "Missing prepared Lean network config at $NETWORK_CONFIG" >&2
    exit 1
  fi
  FLAGS+=(--lean-config "$NETWORK_CONFIG")
fi

if [ -n "$VALIDATOR_REGISTRY" ]; then
  if [ ! -f "$VALIDATOR_REGISTRY" ]; then
    echo "Missing prepared Lean validator registry at $VALIDATOR_REGISTRY" >&2
    exit 1
  fi
  FLAGS+=(--validator-registry "$VALIDATOR_REGISTRY" --node-id "$NODE_ID")
fi

export RUST_LOG="${RUST_LOG:-info}"

# leanMultisig prover: defaults to /usr/local/bin/ethean-prover beside the node;
# HIVE_ETHEAN_PROVER_BIN overrides the path.
if [ -n "${HIVE_ETHEAN_PROVER_BIN:-}" ]; then
  export ETHEAN_PROVER_BIN="$HIVE_ETHEAN_PROVER_BIN"
fi

exec "$ETHEAN_BIN" "${FLAGS[@]}"
