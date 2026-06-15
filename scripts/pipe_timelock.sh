#!/usr/bin/env bash
# ITS-net timelock pipe demo (low epoch count — demo only).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ITS_NET_BIN:-$ROOT/target/release/its-net}"

if [[ ! -x "$BIN" ]]; then
  echo "Build: cd its_net_cli && cargo build --release" >&2
  exit 1
fi

MSG="ITS-net timelock pipe $(date +%s)"
EPOCHS="${PIPE_DEMO_EPOCHS:-25}"

OUT=$(
  printf '%s' "$MSG" | "$BIN" time-lock -f - -o - -e "$EPOCHS" 2>/dev/null \
    | "$BIN" time-unlock -p - -o - 2>/dev/null
)

[[ "$OUT" == "$MSG" ]] || {
  echo "timelock pipe failed" >&2
  exit 1
}

echo "OK: time-lock -f - -o - | time-unlock -p - -o - (epochs=$EPOCHS)"
