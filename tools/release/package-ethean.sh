#!/usr/bin/env bash
# Package the ethean node and its leanMultisig prover for one rustc target.
# Usage: tools/release/package-ethean.sh <version> <target> [dist_dir]
# Writes dist/ethean-v<version>-<target>.{tar.gz|zip} and a matching .sha256 file.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${1:?version required (e.g. 0.1.48)}"
TARGET="${2:?rustc target required}"
DIST="${3:-$ROOT/dist}"

case "$TARGET" in
  *-pc-windows-*) EXT=".exe"; ARCHIVE_EXT="zip" ;;
  *) EXT=""; ARCHIVE_EXT="tar.gz" ;;
esac
BINARIES=("ethean${EXT}" "ethean-prover${EXT}")

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
for name in "${BINARIES[@]}"; do
  BIN=""
  for candidate in "$ROOT/target/${TARGET}/release/${name}" "$ROOT/target/release/${name}"; do
    if [[ -f "$candidate" ]]; then
      BIN="$candidate"
      break
    fi
  done
  if [[ -z "$BIN" ]]; then
    echo "error: ${name} not built for target=${TARGET}" >&2
    exit 1
  fi
  cp "$BIN" "$STAGE/$name"
  if [[ -z "$EXT" ]] && command -v strip >/dev/null 2>&1; then
    strip "$STAGE/$name" || true
  fi
done

mkdir -p "$DIST"
BASE="ethean-v${VERSION}-${TARGET}"
OUT_ARCHIVE="$DIST/${BASE}.${ARCHIVE_EXT}"
if [[ "$ARCHIVE_EXT" == "zip" ]]; then
  (cd "$STAGE" && python3 - "$OUT_ARCHIVE" "${BINARIES[@]}" <<'PY'
import sys, zipfile
with zipfile.ZipFile(sys.argv[1], "w", zipfile.ZIP_DEFLATED) as z:
    for name in sys.argv[2:]:
        z.write(name, name)
PY
  )
else
  tar -czf "$OUT_ARCHIVE" -C "$STAGE" "${BINARIES[@]}"
fi

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$DIST" && sha256sum "$(basename "$OUT_ARCHIVE")" > "${BASE}.sha256")
else
  (cd "$DIST" && shasum -a 256 "$(basename "$OUT_ARCHIVE")" > "${BASE}.sha256")
fi

echo "wrote $OUT_ARCHIVE"
echo "wrote $DIST/${BASE}.sha256"
