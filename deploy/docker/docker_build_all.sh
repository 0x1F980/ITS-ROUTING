#!/usr/bin/env bash
# Build local Docker images for constitution CLIs (skeleton — extend per repo Dockerfile).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ECO="$(cd "$ROOT/.." && pwd)"
KM="${ITS_KM_DIR:-$ECO/ITS-KeyManagement}"

echo "=== Building its-routing:local ==="
docker build -t its-routing:local "$ROOT"

if [[ -f "$KM/Dockerfile" ]]; then
  echo "=== Building its-km:local ==="
  docker build -t its-km:local "$KM"
else
  echo "SKIP: $KM/Dockerfile not found"
fi

if [[ -f "$ECO/ITS-asymmetric/Dockerfile" ]]; then
  echo "=== Building its-asymmetric:local ==="
  docker build -t its-asymmetric:local "$ECO/ITS-asymmetric"
fi

echo "docker_build_all.sh: done"
echo "  docker compose -f $ROOT/deploy/docker/docker-compose.yml up pool-mirror"
