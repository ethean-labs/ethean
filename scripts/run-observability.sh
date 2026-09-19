#!/usr/bin/env bash
# Start Prometheus + Grafana for Ethean long-run metrics.
set -euo pipefail

if ! command -v docker >/dev/null 2>&1; then
  cat <<'EOF' >&2
ERROR: 'docker' is not on PATH.

Grafana (:3000) and Prometheus (:9090) are NOT started by:
  ethean start --until-signal --network …

Install Docker Engine/Desktop, then re-run this script.
Meanwhile the node scrape endpoint is only:
  http://127.0.0.1:9100/metrics
EOF
  exit 1
fi

cd "$(dirname "$0")/../deploy/observability"
docker compose up -d
echo "Grafana: http://localhost:3000  Prometheus: http://localhost:9090"
echo "Scrape target expects ethean on host :9100 (ethean start --until-signal)"
