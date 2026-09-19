#!/usr/bin/env bash
# Local finality long-run: head advances without public bootnodes.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build -p ethean --release

NETWORK="${NETWORK:-pq-devnet-4}"
VALIDATORS="${VALIDATORS:-4}"
EXTRA=()
if [[ "${METRICS_STACK:-0}" == "1" ]]; then
  EXTRA+=(--metrics)
fi

echo "ethean start --until-signal --network $NETWORK --validators $VALIDATORS ${EXTRA[*]:-}"
echo "Health: http://127.0.0.1:9100/healthz  Metrics: http://127.0.0.1:9100/metrics"
exec ethean start --until-signal --network "$NETWORK" --validators "$VALIDATORS" "${EXTRA[@]}"
