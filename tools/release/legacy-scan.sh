#!/usr/bin/env bash
# Legacy / forbidden symbol scan for releases.
# Hard patterns (BLS crates in active code) fail the scan; soft patterns are
# reported only. road-to/** migration records are exempt.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
OUT="$ROOT/artifacts/phase-13/legacy-scan"
mkdir -p "$OUT"

scan() { # scan <report> <dirs...> -- <patterns...>
  local report="$1"; shift
  local dirs=()
  while [ "$1" != "--" ]; do [ -e "$1" ] && dirs+=("$1"); shift; done
  shift
  : > "$report"
  for pattern in "$@"; do
    grep -rnE --exclude-dir=road-to --exclude-dir=target "$pattern" "${dirs[@]}" >> "$report" 2>/dev/null || true
  done
  [ -s "$report" ] || echo CLEAN > "$report"
}

scan "$OUT/hits.txt" crates bin tests examples docs deploy tools -- \
  'panro' 'Panro' '/eth2/' 'NetworkManager' 'mock.?gossip'
scan "$OUT/hard-hits.txt" crates bin -- '\bblst\b' '\bblstrs\b' '\bbls12_381\b'

if [ "$(cat "$OUT/hard-hits.txt")" != CLEAN ]; then
  echo "HARD legacy scan FAILED: see $OUT/hard-hits.txt"
  exit 1
fi
if [ "$(cat "$OUT/hits.txt")" = CLEAN ]; then
  echo "Legacy scan CLEAN (soft+hard)"
else
  echo "Soft legacy scan found $(wc -l < "$OUT/hits.txt") hit lines: see $OUT/hits.txt (hard CLEAN)"
fi
