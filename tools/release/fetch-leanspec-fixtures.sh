#!/usr/bin/env bash
# Download and verify leanSpec production-scheme fixtures into a local cache.
# Digests come from spec/fixtures/phase-00/manifest.toml; nothing is committed.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

MANIFEST="$ROOT/spec/fixtures/phase-00/manifest.toml"
CACHE="$ROOT/.cache/leanspec-fixtures"
ARCHIVE="$CACHE/fixtures-prod-scheme.tar.gz"
EXTRACTED="$CACHE/extracted"

toml_value() {
  tr -d '\r' < "$MANIFEST" | sed -nE "s/^[[:space:]]*$1[[:space:]]*=[[:space:]]*\"?([^\"]+)\"?[[:space:]]*$/\1/p" | head -n1
}

[ -f "$MANIFEST" ] || { echo "missing manifest: $MANIFEST" >&2; exit 1; }
URL="$(toml_value browser_download_url)"
WANT_SHA="$(toml_value sha256 | tr 'A-F' 'a-f')"
WANT_SIZE="$(toml_value size_bytes)"
if [ -z "$URL" ] || [ -z "$WANT_SHA" ] || [ -z "$WANT_SIZE" ]; then
  echo "manifest.toml missing browser_download_url / sha256 / size_bytes" >&2
  exit 1
fi

verified() {
  [ -f "$ARCHIVE" ] || return 1
  [ "$(stat -c %s "$ARCHIVE")" = "$WANT_SIZE" ] || return 1
  [ "$(sha256sum "$ARCHIVE" | awk '{print $1}')" = "$WANT_SHA" ]
}

mkdir -p "$CACHE"
if verified; then
  echo "Archive already verified at $ARCHIVE"
else
  echo "Downloading $URL"
  curl --fail --location --silent --show-error --output "$ARCHIVE" "$URL"
  verified || { echo "size or sha256 mismatch for $ARCHIVE" >&2; exit 1; }
  echo "Verified sha256 $WANT_SHA ($WANT_SIZE bytes)"
fi

rm -rf "$EXTRACTED"
mkdir -p "$EXTRACTED"
tar -xzf "$ARCHIVE" -C "$EXTRACTED"
[ -d "$EXTRACTED/fixtures/consensus" ] || { echo "expected fixtures/consensus in archive" >&2; exit 1; }

echo
echo "Ready. Point consumers at:"
echo "  export ETHEAN_LEANSPEC_FIXTURES=\"$EXTRACTED\""
echo "Then: cargo test -p ethean-spec-fixtures"
