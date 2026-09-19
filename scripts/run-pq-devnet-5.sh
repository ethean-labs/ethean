#!/usr/bin/env bash
# Ready-path runner for pq-devnet-5 (needs operator bootnodes / fork digest).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build -p ethean --release
exec ethean start --until-signal --network pq-devnet-5 --metrics "$@"
