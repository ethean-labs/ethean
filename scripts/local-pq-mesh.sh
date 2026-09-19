#!/usr/bin/env bash
# Local private PQ mesh (2 peers) — Unix
#
# Ream-style flow: start peer A, write dialable multiaddr as the mesh
# nodes list, peer B dials it (Ethean equivalent of nodes.yaml).
#
# Usage (from repo root):
#   ./scripts/local-pq-mesh.sh
#   NETWORK=pq-devnet-4 PEER_B_TICKS=15 ./scripts/local-pq-mesh.sh

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

NETWORK="${NETWORK:-pq-devnet-4}"
PEER_B_TICKS="${PEER_B_TICKS:-12}"
WAIT_DIALABLE_SEC="${WAIT_DIALABLE_SEC:-45}"

MESH_DIR="$ROOT/target/local-pq-mesh"
mkdir -p "$MESH_DIR"
LOG_A="$MESH_DIR/peer-a.log"
LOG_B="$MESH_DIR/peer-b.log"
NODES_FILE="$MESH_DIR/nodes.multiaddrs"

find_ethean() {
  if [[ -x "$ROOT/target/release/ethean" ]]; then
    echo "$ROOT/target/release/ethean"
  elif [[ -x "$ROOT/target/debug/ethean" ]]; then
    echo "$ROOT/target/debug/ethean"
  elif command -v ethean >/dev/null 2>&1; then
    command -v ethean
  else
    return 1
  fi
}

echo "==> Building ethean (release)…"
cargo build -p ethean --release
ETHEAN="$(find_ethean)" || { echo "ethean binary not found"; exit 1; }
echo "==> Using $ETHEAN"

rm -f "$LOG_A" "$LOG_B" "$NODES_FILE"
PEER_A_PID=""
cleanup() {
  if [[ -n "${PEER_A_PID}" ]] && kill -0 "$PEER_A_PID" 2>/dev/null; then
    kill "$PEER_A_PID" 2>/dev/null || true
    wait "$PEER_A_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

echo "==> Starting peer A (--until-signal --network $NETWORK)…"
export RUST_LOG=info
"$ETHEAN" start --until-signal --network "$NETWORK" >"$LOG_A" 2>&1 &
PEER_A_PID=$!

dialable=""
deadline=$((SECONDS + WAIT_DIALABLE_SEC))
while (( SECONDS < deadline )); do
  if ! kill -0 "$PEER_A_PID" 2>/dev/null; then
    echo "peer A exited early; see $LOG_A"
    exit 1
  fi
  if grep -qoE 'dialable=[^[:space:]]+' "$LOG_A" 2>/dev/null; then
    dialable="$(grep -oE 'dialable=[^[:space:]]+' "$LOG_A" | tail -n1 | sed 's/^dialable=//')"
    break
  fi
  sleep 0.4
done

if [[ -z "$dialable" ]]; then
  echo "timed out waiting for peer A dialable= (see $LOG_A)"
  exit 1
fi

boot="$(echo "$dialable" | sed -E 's#^/ip4/[^/]+/#/ip4/127.0.0.1/#')"
cat >"$NODES_FILE" <<EOF
# Auto-generated local private mesh peer list (Ethean multiaddrs).
# Equivalent role to lean-quickstart / Ream genesis nodes.yaml for this run.
# Peer A dialable (original): $dialable
$boot
EOF

echo "==> Wrote mesh nodes file: $NODES_FILE"
echo "    bootnode: $boot"

echo "==> Starting peer B (dial mesh, --ticks $PEER_B_TICKS --wall-clock)…"
set +e
"$ETHEAN" start \
  --network "$NETWORK" \
  --ticks "$PEER_B_TICKS" \
  --wall-clock \
  --bootnodes "$boot" >"$LOG_B" 2>&1
exit_b=$?
set -e

dial_ok=0
status_ok=0
grep -q 'dialed bootnode\|ConnectionEstablished' "$LOG_B" && dial_ok=1 || true
grep -q 'Status handshake' "$LOG_B" "$LOG_A" && status_ok=1 || true

echo ""
echo "=== Local private mesh result ==="
echo "nodes file : $NODES_FILE"
echo "peer A log : $LOG_A"
echo "peer B log : $LOG_B"
echo "peer B exit: $exit_b"
echo "dial seen  : $dial_ok"
echo "status seen: $status_ok"

if [[ "$exit_b" -ne 0 ]]; then
  exit "$exit_b"
fi
if [[ "$dial_ok" -ne 1 ]]; then
  echo "WARN: bootnode dial line not found; check peer B logs."
  exit 2
fi
echo "OK: private mesh dial path exercised."
