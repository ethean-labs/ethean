#!/usr/bin/env bash
# Package the ethean binary for one rustc target into dist/.
# Usage: tools/release/package-ethean.sh <version> <target> [dist_dir]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${1:?version required (e.g. 0.1.47)}"
TARGET="${2:?rustc target required}"
DIST="${3:-$ROOT/dist}"

case "$TARGET" in
  *-pc-windows-*) EXT=".exe"; ARCHIVE_EXT="zip" ;;
  *) EXT=""; ARCHIVE_EXT="tar.gz" ;;
esac

BIN_NAME="ethean${EXT}"
CANDIDATES=(
  "$ROOT/target/${TARGET}/release/${BIN_NAME}"
  "$ROOT/target/release/${BIN_NAME}"
)
BIN=""
for c in "${CANDIDATES[@]}"; do
  if [[ -f "$c" ]]; then
    BIN="$c"
    break
  fi
done
if [[ -z "$BIN" ]]; then
  echo "error: binary not found for target=${TARGET}" >&2
  printf '  looked: %s\n' "${CANDIDATES[@]}" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp "$BIN" "$STAGE/$BIN_NAME"
# Strip on ELF/Mach-O when strip exists (no-op on Windows path).
if command -v strip >/dev/null 2>&1 && [[ "$EXT" != ".exe" ]]; then
  strip "$STAGE/$BIN_NAME" || true
fi

mkdir -p "$DIST"
BASE="ethean-v${VERSION}-${TARGET}"
OUT_ARCHIVE="$DIST/${BASE}.${ARCHIVE_EXT}"

(
  cd "$STAGE"
  if [[ "$ARCHIVE_EXT" == "zip" ]]; then
    if command -v zip >/dev/null 2>&1; then
      zip -q "$OUT_ARCHIVE" "$BIN_NAME"
    else
      python3 - <<PY
import zipfile
z = zipfile.ZipFile(r"$OUT_ARCHIVE", "w", zipfile.ZIP_DEFLATED)
z.write("$BIN_NAME", "$BIN_NAME")
z.close()
PY
    fi
  else
    tar -czf "$OUT_ARCHIVE" "$BIN_NAME"
  fi
)

# SHA256 beside the archive (portable).
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$DIST" && sha256sum "$(basename "$OUT_ARCHIVE")" > "${BASE}.sha256")
elif command -v shasum >/dev/null 2>&1; then
  (cd "$DIST" && shasum -a 256 "$(basename "$OUT_ARCHIVE")" > "${BASE}.sha256")
else
  python3 - <<PY
import hashlib, pathlib
p = pathlib.Path(r"$OUT_ARCHIVE")
h = hashlib.sha256(p.read_bytes()).hexdigest()
pathlib.Path(r"$DIST/${BASE}.sha256").write_text(f"{h}  {p.name}\n")
PY
fi

echo "wrote $OUT_ARCHIVE"
echo "wrote $DIST/${BASE}.sha256"
