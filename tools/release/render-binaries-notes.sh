#!/usr/bin/env bash
# Render a Lighthouse-style Binaries markdown table for release notes.
# Usage: tools/release/render-binaries-notes.sh <version> <dist_dir>
set -euo pipefail

VERSION="${1:?version required}"
DIST="${2:?dist dir required}"
REPO_URL="${REPO_URL:-https://github.com/pamenarti/ethean}"
TAG="v${VERSION}"
BASE_URL="${REPO_URL}/releases/download/${TAG}"

asset() {
  local name="$1"
  if [[ -f "$DIST/$name" ]]; then
    echo "$name"
  else
    echo ""
  fi
}

row() {
  local system="$1" arch="$2" target="$3" ext="$4"
  local file="ethean-v${VERSION}-${target}.${ext}"
  local sha="ethean-v${VERSION}-${target}.sha256"
  if [[ ! -f "$DIST/$file" ]]; then
    return 0
  fi
  local file_link="[${file}](${BASE_URL}/${file})"
  local sha_link="—"
  if [[ -f "$DIST/$sha" ]]; then
    sha_link="[${sha}](${BASE_URL}/${sha})"
  fi
  printf '| %s | `%s` | %s | %s |\n' "$system" "$arch" "$file_link" "$sha_link"
}

cat <<EOF
## Binaries

Pre-built archives for this tag, each containing \`ethean\` and \`ethean-prover\`
(the leanMultisig prover aggregators and proposers need; keep it next to \`ethean\`).
Default features include QUIC (\`libp2p-quic\`). XMSS is native in
\`ethean-crypto\`; aggregate proofs use leanMultisig at the pq-devnet-4 leanVM pin.

Verify downloads with the matching \`.sha256\` file:

\`\`\`bash
sha256sum -c ethean-v${VERSION}-<target>.sha256
\`\`\`

| System | Architecture | Binary | SHA256 |
| --- | --- | --- | --- |
EOF

row "Windows" "x86_64" "x86_64-pc-windows-msvc" "zip"
row "Linux" "x86_64" "x86_64-unknown-linux-gnu" "tar.gz"
row "Linux" "aarch64" "aarch64-unknown-linux-gnu" "tar.gz"
row "macOS" "aarch64" "aarch64-apple-darwin" "tar.gz"
row "macOS" "x86_64" "x86_64-apple-darwin" "tar.gz"

cat <<EOF

Source archives are attached automatically by GitHub. Build locally with:

\`\`\`bash
cargo build -p ethean -p ethean-prover --release --locked
\`\`\`

See [docs/release/binaries.md](${REPO_URL}/blob/master/docs/release/binaries.md).
EOF
