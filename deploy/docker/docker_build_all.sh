#!/usr/bin/env bash
# Build local Docker images for constitution CLIs (prebuilds v2).
# Builds its-routing + its-km release images; optional its-asymmetric.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ECO="$(cd "$ROOT/.." && pwd)"
KM="${ITS_KM_DIR:-$ECO/ITS-KeyManagement}"
ASY="${ITS_ASYMMETRIC_DIR:-$ECO/ITS-asymmetric}"
COMPOSE="$ROOT/deploy/docker/docker-compose.yml"

echo "=== Building its-routing:local ==="
docker build -t its-routing:local "$ROOT"

if [[ -f "$KM/Dockerfile" ]]; then
  echo "=== Building its-km:local ==="
  docker build -t its-km:local "$KM"
  echo "=== Building its-km-compose:local (alpine sidecar) ==="
  docker build -t its-km-compose:local -f "$ROOT/deploy/docker/Dockerfile.its-km-compose" "$ROOT/deploy/docker"
else
  echo "SKIP: $KM/Dockerfile not found (set ITS_KM_DIR)" >&2
  exit 1
fi

if [[ -f "$ASY/Dockerfile" ]]; then
  echo "=== Building its-asymmetric:local ==="
  docker build -t its-asymmetric:local "$ASY"
else
  echo "SKIP: $ASY/Dockerfile not found (optional)"
fi

echo ""
echo "docker_build_all.sh: done"
echo ""
echo "Smoke stack:"
echo "  docker compose -f $COMPOSE up -d"
echo "  curl -sf 'http://127.0.0.1:8787/pool/cells?from=0' >/dev/null && echo pool-mirror OK"
echo ""
echo "Exec CLIs:"
echo "  docker compose -f $COMPOSE exec its-routing its-routing --help"
echo "  docker compose -f $COMPOSE exec its-km its-km --help"
echo ""
echo "Prod gate (no silent file fallback in pipes):"
echo "  ITS_PROD_GATE=1 docker compose -f $COMPOSE up -d pool-mirror"
