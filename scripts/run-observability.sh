#!/usr/bin/env bash
# Start Prometheus + Grafana for Ethean long-run metrics.
set -euo pipefail
cd "$(dirname "$0")/../deploy/observability"
docker compose up -d
echo "Grafana: http://localhost:3000  Prometheus: http://localhost:9090"
echo "Scrape target expects ethean on host :9100 (ethean start --until-signal)"
