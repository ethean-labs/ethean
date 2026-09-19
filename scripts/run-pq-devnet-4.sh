#!/usr/bin/env bash
# Run Ethean against the operational pq-devnet-4 label.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build -p ethean --release
exec ethean start --until-signal --network pq-devnet-4 --metrics "$@"
