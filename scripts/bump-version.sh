#!/usr/bin/env bash
# Bump Ethean workspace version (patch++ ; 0.1.99 -> 0.2.0)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION_FILE="$ROOT/VERSION"
CARGO_TOML="$ROOT/Cargo.toml"
CARGO_LOCK="$ROOT/Cargo.lock"

raw="$(tr -d '[:space:]' <"$VERSION_FILE")"
if [[ ! "$raw" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  echo "VERSION must be MAJOR.MINOR.PATCH (got '$raw')" >&2
  exit 1
fi

major="${BASH_REMATCH[1]}"
minor="${BASH_REMATCH[2]}"
patch="${BASH_REMATCH[3]}"
prev="${major}.${minor}.${patch}"

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

if [[ -f "$CARGO_LOCK" ]]; then
  # Only ethean* package stanzas — never blanket-replace (breaks crates.io pins).
  python3 - "$CARGO_LOCK" "$prev" "$next" <<'PY'
import re, sys
path, prev, nxt = sys.argv[1], sys.argv[2], sys.argv[3]
raw = open(path, "rb").read()
if raw.startswith(b"\xef\xbb\xbf"):
    raw = raw[3:]
text = raw.decode("utf-8")
pat = rf'(\[\[package\]\]\nname = "ethean[^"]*"\n)version = "{re.escape(prev)}"'
text2, n = re.subn(pat, rf'\1version = "{nxt}"', text)
if n == 0:
    raise SystemExit(f"Cargo.lock has no ethean packages at {prev} to bump")
open(path, "wb").write(text2.encode("utf-8"))
print(f"Bumped version to {nxt} (VERSION + Cargo.toml + Cargo.lock ethean* x{n})")
PY
else
  echo "Bumped version to $next (VERSION + Cargo.toml)"
fi
