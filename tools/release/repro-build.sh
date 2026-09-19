#!/usr/bin/env bash
# Reproducible release build (Unix). Usage: tools/release/repro-build.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
OUT="$ROOT/artifacts/phase-13/repro"
mkdir -p "$OUT"
cargo build --release --locked \
  -p ethean-primitives -p ethean-profile -p ethean-ssz -p ethean-types \
  -p ethean-crypto -p ethean-genesis -p ethean-transition -p ethean-fork-choice \
  -p ethean-validator -p ethean-network-wire -p ethean-network \
  -p ethean-storage -p ethean-sync -p ethean-rpc -p ethean-metrics
{
  echo "recorded_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "git_head=$(git rev-parse HEAD)"
  echo "lock_sha256=$(sha256sum Cargo.lock | awk '{print $1}')"
} > "$OUT/build-meta.txt"
echo "Wrote $OUT/build-meta.txt"
