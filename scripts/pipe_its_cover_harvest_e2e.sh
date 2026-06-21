#!/usr/bin/env bash
# CoverTransport E2E: pool + entropy_sources cover harvest every epoch.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/pipe_pool_common.sh
source "$ROOT/scripts/lib/pipe_pool_common.sh"

pipe_pool_init "$ROOT" "its_cover_e2e"
COVER_LOG="$TMP/cover.log"

pipe_pool_keygen "$TMP/bob"
MSG="cover-harvest e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"
pipe_pool_encrypt "$TMP/bob" "$TMP/msg.txt" "$TMP/msg.wire"

# Local cover source (mock HTTP telemetry)
python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9192 --store-dir "$TMP/cover-store" &
COVER_PID=$!
sleep 1.5
trap 'kill $COVER_PID 2>/dev/null; rm -rf "$TMP"' EXIT

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$POOL"
cell_size_L = 4096
epoch_interval_ms = 50
sss_k = 2
sss_n = 3

[aeh]
entropy_sources = ["http://127.0.0.1:9192/pool/cells?from=0"]
EOF

"$ROUTING" -c "$TMP/pool.toml" client-send --pool --file "$TMP/msg.wire" \
  --ratchet-seed-file "$TMP/ratchet.seed" 2>&1 | tee "$COVER_LOG"

"$ROUTING" -c "$TMP/pool.toml" client-receive --pool --continuous \
  --ratchet-seed-file "$TMP/ratchet.seed" --out "$TMP/recv.wire" --timeout-secs 15 2>&1 | tee -a "$COVER_LOG"

if ! grep -q "cover_sources=1" "$COVER_LOG"; then
  echo "cover pipe: expected Cover harvest log line" >&2
  exit 1
fi

pipe_pool_decrypt "$TMP/bob" "$TMP/recv.wire" "$TMP/out.txt"
[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_cover_harvest_e2e.sh: OK"
