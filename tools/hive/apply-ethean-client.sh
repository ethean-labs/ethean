#!/usr/bin/env bash
# Apply Ethean Lean client registration onto an ethereum/hive checkout.
#
# Usage:
#   ./tools/hive/apply-ethean-client.sh /path/to/hive
#
# Idempotent: safe to re-run. Does not invent digests or bootnodes.
set -euo pipefail

HIVE_ROOT="${1:-}"
if [[ -z "$HIVE_ROOT" || ! -d "$HIVE_ROOT" ]]; then
  echo "usage: $0 /path/to/ethereum/hive" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DROPIN="$REPO_ROOT/docker/hive/upstream-clients-ethean"
DEST="$HIVE_ROOT/clients/ethean"

if [[ ! -d "$DROPIN" ]]; then
  echo "missing drop-in at $DROPIN" >&2
  exit 1
fi

mkdir -p "$DEST"
cp -f "$DROPIN/Dockerfile" "$DEST/Dockerfile"
cp -f "$DROPIN/Dockerfile.git" "$DEST/Dockerfile.git"
cp -f "$DROPIN/ethean.sh" "$DEST/ethean.sh"
cp -f "$DROPIN/hive.yaml" "$DEST/hive.yaml"
cp -f "$DROPIN/validators.yaml" "$DEST/validators.yaml"
chmod +x "$DEST/ethean.sh"
echo "copied clients/ethean drop-in"

DEVNET4="$HIVE_ROOT/simulators/lean/clients/devnet4.yaml"
DEVNET5="$HIVE_ROOT/simulators/lean/clients/devnet5.yaml"
MATRIX="$HIVE_ROOT/simulators/lean/config/lean-devnets.txt"
UTIL="$HIVE_ROOT/simulators/lean/src/utils/util.rs"
PREP="$HIVE_ROOT/simulators/lean/helper/prepare_lean_client_assets.py"

append_client_yaml() {
  local file="$1"
  local nametag="$2"
  if grep -qE '^- client: ethean$' "$file"; then
    echo "ok: $file already lists ethean"
    return
  fi
  printf '\n- client: ethean\n  nametag: %s\n' "$nametag" >>"$file"
  echo "appended ethean to $file"
}

append_client_yaml "$DEVNET4" "devnet4"
append_client_yaml "$DEVNET5" "devnet5"

if grep -qE '^ethean=devnet4,devnet5$' "$MATRIX"; then
  echo "ok: lean-devnets.txt already lists ethean"
else
  printf '\nethean=devnet4,devnet5\n' >>"$MATRIX"
  echo "appended ethean matrix row to lean-devnets.txt"
fi

# util.rs candidate list (lean_client_kind) — multiaddr client like ream.
if grep -q '"ethean"' "$UTIL"; then
  echo "ok: util.rs already mentions ethean"
else
  # Insert after "ream", before "gean" when that ordering exists.
  if grep -q '"ream",' "$UTIL" && grep -q '"gean",' "$UTIL"; then
    python3 - "$UTIL" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
needle = '"ream",\n'
insert = '"ream",\n            "ethean",\n'
if '"ethean"' in text:
    pass
elif needle in text:
    path.write_text(text.replace(needle, insert, 1))
else:
    raise SystemExit("util.rs: could not find ream candidate to patch")
PY
    echo "patched util.rs lean client candidates"
  else
    echo "WARN: util.rs layout unexpected; add \"ethean\" to lean_client_kind candidates manually" >&2
  fi
fi

# prepare_lean_client_assets.py — ream-shaped dual-key writer for ethean.
if grep -q '"ethean"' "$PREP"; then
  echo "ok: prepare_lean_client_assets.py already mentions ethean"
else
  python3 - "$PREP" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
if '"ethean"' in text:
    raise SystemExit(0)
# SUPPORTED_CLIENTS set
old = 'SUPPORTED_CLIENTS = {\n "ethlambda",'
new = 'SUPPORTED_CLIENTS = {\n "ethean",\n "ethlambda",'
if old not in text:
    old = 'SUPPORTED_CLIENTS = {\n    "ethlambda",'
    new = 'SUPPORTED_CLIENTS = {\n    "ethean",\n    "ethlambda",'
if old not in text:
    raise SystemExit("prepare script: SUPPORTED_CLIENTS block not found")
text = text.replace(old, new, 1)
# Treat ethean like ream in config + asset writers.
replacements = [
    ('if CLIENT_KIND == "ream":', 'if CLIENT_KIND in {"ream", "ethean"}:'),
]
for a, b in replacements:
    text = text.replace(a, b)
path.write_text(text)
print("patched prepare_lean_client_assets.py")
PY
fi

echo
echo "Done. Next (outside this script):"
echo "  1. Ensure ghcr.io/ethean-labs/ethean:devnet5 exists (CI publish)."
echo "  2. From hive root: ./hive --sim lean --client-file simulators/lean/clients/devnet5.yaml --client ethean --docker.output"
echo "  3. Open ethereum/hive PR with tools/hive/PR_BODY.md"
