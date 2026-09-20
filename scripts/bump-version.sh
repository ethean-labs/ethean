#!/usr/bin/env bash
# Bump Ethean workspace version (patch++ ; 0.1.99 -> 0.2.0)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION_FILE="$ROOT/VERSION"
CARGO_TOML="$ROOT/Cargo.toml"

raw="$(tr -d '[:space:]' <"$VERSION_FILE")"
if [[ ! "$raw" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  echo "VERSION must be MAJOR.MINOR.PATCH (got '$raw')" >&2
  exit 1
fi

major="${BASH_REMATCH[1]}"
minor="${BASH_REMATCH[2]}"
patch="${BASH_REMATCH[3]}"

patch=$((patch + 1))
if (( patch > 99 )); then
  patch=0
  minor=$((minor + 1))
  if (( minor > 99 )); then
    minor=0
    major=$((major + 1))
  fi
fi

next="${major}.${minor}.${patch}"
printf '%s\n' "$next" >"$VERSION_FILE"

# Only the first version = "..." in the file (workspace.package).
awk -v ver="$next" '
  BEGIN { done = 0 }
  /^version = "[0-9]+\.[0-9]+\.[0-9]+"/ && !done {
    print "version = \"" ver "\""
    done = 1
    next
  }
  { print }
' "$CARGO_TOML" >"$CARGO_TOML.tmp"
mv "$CARGO_TOML.tmp" "$CARGO_TOML"

echo "Bumped version to $next (VERSION + Cargo.toml)"
