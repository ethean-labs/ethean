#!/usr/bin/env bash
# SBOM stub — record Cargo.lock hash until cargo-cyclonedx is wired in CI.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
OUT="$ROOT/artifacts/phase-13/sbom"
mkdir -p "$OUT"
echo "Install cargo-cyclonedx for full SBOM; lock hash recorded as interim fingerprint." > "$OUT/README.txt"
sha256sum Cargo.lock | awk '{print $1}' > "$OUT/cargo.lock.sha256"
echo "Wrote $OUT/cargo.lock.sha256"
