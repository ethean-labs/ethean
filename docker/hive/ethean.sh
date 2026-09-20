#!/usr/bin/env bash
# Hive entrypoint for Ethean Lean Consensus Client.
# Maps ethereum/hive Lean simulator env vars onto `ethean start`.
set -euo pipefail

DEVNET_LABEL="${HIVE_LEAN_DEVNET_LABEL:-devnet5}"
BOOTNODES="${HIVE_BOOTNODES:-}"
FORK_DIGEST="${HIVE_FORK_DIGEST:-${HIVE_LEAN_FORK_DIGEST:-}}"
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

# Honesty: Hive prepares Ream-style config.yaml + validators.yaml under
# HIVE_LEAN_NETWORK_CONFIG / HIVE_LEAN_VALIDATOR_REGISTRY_PATH. Ethean does not
# yet consume those files; network label + bootnodes/fork-digest env are used.
if [ -n "${HIVE_LEAN_NETWORK_CONFIG:-}" ]; then
  echo "note: HIVE_LEAN_NETWORK_CONFIG is set but ignored by this Ethean scaffold" >&2
fi

export RUST_LOG="${RUST_LOG:-info}"

exec "$ETHEAN_BIN" "${FLAGS[@]}"
