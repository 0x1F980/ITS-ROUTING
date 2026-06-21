#!/usr/bin/env bash
# ITS-routing timelock pipe demo (low epoch count — demo only).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ITS_NET_BIN:-$ROOT/target/release/its-routing}"

# Timelock is an optional ridge — ensure release binary includes it for M20.
if ! printf '' | "$BIN" time-lock -f - -o - -e 1 2>&1 | grep -qv 'without.*timelock'; then
  echo "Building its-routing with timelock feature..." >&2
  (cd "$ROOT" && cargo build -p its_routing --release --features timelock --quiet)
  BIN="$ROOT/target/release/its-routing"
fi

if printf '' | "$BIN" time-lock -f - -o - -e 1 2>&1 | grep -q 'without.*timelock'; then
  echo "its-routing built without timelock — rebuild: cargo build -p its_routing --features timelock" >&2
  exit 1
fi

MSG="ITS-routing timelock pipe $(date +%s)"
EPOCHS="${PIPE_DEMO_EPOCHS:-25}"

OUT=$(
  printf '%s' "$MSG" | "$BIN" time-lock -f - -o - -e "$EPOCHS" 2>/dev/null \
    | "$BIN" time-unlock -p - -o - 2>/dev/null
)
# time-unlock may print config notice on stdout — compare payload line only.
OUT="${OUT##*$'\n'}"

[[ "$OUT" == "$MSG" ]] || {
  echo "timelock pipe failed" >&2
  exit 1
}

echo "OK: time-lock -f - -o - | time-unlock -p - -o - (epochs=$EPOCHS)"
