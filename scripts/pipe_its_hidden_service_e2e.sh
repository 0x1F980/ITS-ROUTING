#!/usr/bin/env bash
# Hidden service E2E skeleton (Fase 4): extends M19 SOCKS pool gate.
# Full static-site publish via compose profile is operator follow-up.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "== hidden service skeleton: delegate to M19 SOCKS pool gate =="
echo "Reference stack: deploy/hidden-service/README.md"
echo "Operator guide: ITS_HIDDEN_SERVICE.md"

"$ROOT/scripts/pipe_its_socks_pool_e2e.sh"

echo "pipe_its_hidden_service_e2e.sh: OK (skeleton — M19 v2 passed)"
