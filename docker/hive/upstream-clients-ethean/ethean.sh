#!/bin/bash

# ethereum/hive clients/ethean entrypoint. Maps the Lean simulator's HIVE_*
# environment onto `ethean start`. Same asset contract as clients/ream: the
# prepared config.yaml + validators.yaml (ream shape, hash-sig-keys/ secrets)
# must exist under /tmp/ethean-runtime.

set -euo pipefail

DEVNET_LABEL="${HIVE_LEAN_DEVNET_LABEL:-devnet5}"
NODE_ID="${HIVE_NODE_ID:-ethean_0}"
BOOTNODES="${HIVE_BOOTNODES:-none}"
ASSET_ROOT="/tmp/ethean-runtime"
KEY_FILE=""
NETWORK_CONFIG="${HIVE_LEAN_NETWORK_CONFIG:-$ASSET_ROOT/config.yaml}"
VALIDATOR_REGISTRY_PATH="${HIVE_LEAN_VALIDATOR_REGISTRY_PATH:-$ASSET_ROOT/validators.yaml}"

case "$DEVNET_LABEL" in
    devnet5|devnet4)
        # One published image covers both labels; genesis comes from config.yaml.
        DEFAULT_ETHEAN_BIN="/usr/local/bin/ethean-devnet5"
        ;;
    *)
        echo "Unsupported Lean devnet label: $DEVNET_LABEL" >&2
        exit 1
        ;;
esac

ETHEAN_BIN="${ETHEAN_BIN:-$DEFAULT_ETHEAN_BIN}"

cleanup() {
    if [ -n "$KEY_FILE" ] && [ -f "$KEY_FILE" ]; then
        rm -f "$KEY_FILE"
    fi
}

trap cleanup EXIT

if [ ! -f "$NETWORK_CONFIG" ]; then
    echo "Missing prepared Lean network config at $NETWORK_CONFIG" >&2
    exit 1
fi

if [ ! -f "$VALIDATOR_REGISTRY_PATH" ]; then
    echo "Missing prepared Lean validator registry at $VALIDATOR_REGISTRY_PATH" >&2
    exit 1
fi

# `--network` accepts a path to Hive/quickstart config.yaml (local profile).
# `--ephemeral` is a `start` flag (clap), not a global like ream's `--ephemeral`.
FLAGS=(
    start
    --ephemeral
    --until-signal
    --no-banner
    --network "$NETWORK_CONFIG"
    --validator-registry-path "$VALIDATOR_REGISTRY_PATH"
    --node-id "$NODE_ID"
    --socket-address 0.0.0.0
    --socket-port 9000
    --http-address 0.0.0.0
    --http-port 5052
    --bootnodes "$BOOTNODES"
)

if [ -n "${HIVE_LEAN_FORK_DIGEST:-}" ]; then
    FLAGS+=(--fork-digest "$HIVE_LEAN_FORK_DIGEST")
fi

if [ -n "${HIVE_CHECKPOINT_SYNC_URL:-}" ]; then
    FLAGS+=(--checkpoint-sync-url "$HIVE_CHECKPOINT_SYNC_URL")
fi

if [ "${HIVE_IS_AGGREGATOR:-0}" = "1" ]; then
    FLAGS+=(--is-aggregator)
fi

if [ -n "${HIVE_AGGREGATE_SUBNET_IDS:-}" ]; then
    FLAGS+=(--aggregate-subnet-ids "$HIVE_AGGREGATE_SUBNET_IDS")
fi

if [ -n "${HIVE_ATTESTATION_COMMITTEE_COUNT:-}" ] && [ "$HIVE_ATTESTATION_COMMITTEE_COUNT" != "1" ]; then
    FLAGS+=(--attestation-committee-count "$HIVE_ATTESTATION_COMMITTEE_COUNT")
fi

if [ -n "${HIVE_CLIENT_PRIVATE_KEY:-}" ]; then
    KEY_FILE="$(mktemp /tmp/ethean-node-key.XXXXXX)"
    printf "%s" "$HIVE_CLIENT_PRIVATE_KEY" > "$KEY_FILE"
    FLAGS+=(--node-key "$KEY_FILE")
elif [ -f "$ASSET_ROOT/node.key" ]; then
    FLAGS+=(--node-key "$ASSET_ROOT/node.key")
fi

if [ "${HIVE_METRICS_ENABLED:-0}" = "1" ]; then
    FLAGS+=(--metrics --metrics-address 0.0.0.0 --metrics-port 8080)
fi

# Spec-asset scenarios set HIVE_LEAN_TEST_DRIVER=1; the node reads it.
if [ "${HIVE_LEAN_TEST_DRIVER:-0}" = "1" ]; then
    export HIVE_LEAN_TEST_DRIVER=1
fi

export RUST_LOG="${RUST_LOG:-info}"
# leanMultisig prover beside the node binary (sibling lookup also works).
export ETHEAN_PROVER_BIN="${ETHEAN_PROVER_BIN:-/usr/local/bin/ethean-prover}"

exec "$ETHEAN_BIN" "${FLAGS[@]}"
