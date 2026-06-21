#!/usr/bin/env bash
# Public mirror deploy reference E2E (same as http_pool with deploy path check).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[[ -f "$ROOT/deploy/pool-mirror/pool_mirror_server.py" ]] || {
  echo "deploy/pool-mirror missing" >&2
  exit 1
}
exec "$ROOT/scripts/pipe_its_http_pool_e2e.sh"
