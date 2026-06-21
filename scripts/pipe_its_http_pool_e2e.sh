#!/usr/bin/env bash
# HTTP pool E2E with ITS_PROD_GATE=1 (no file fallback) + deploy/pool-mirror path check.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/pipe_pool_common.sh
source "$ROOT/scripts/lib/pipe_pool_common.sh"

[[ -f "$ROOT/deploy/pool-mirror/pool_mirror_server.py" ]] || {
  echo "deploy/pool-mirror missing" >&2
  exit 1
}

pipe_pool_init "$ROOT" "its_http_pool_e2e"
POOL_FILE="$TMP/file-fallback-should-not-exist"

MIRROR_PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port "$MIRROR_PORT" --store-dir "$TMP/mirror-store" &
MIRROR_PID=$!
trap 'kill ${MIRROR_PID:-} 2>/dev/null; rm -rf "$TMP"' EXIT
for _ in $(seq 1 30); do
  if kill -0 "$MIRROR_PID" 2>/dev/null && python3 -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:${MIRROR_PORT}/pool/cells?from=0', timeout=0.5)" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done
if ! kill -0 "$MIRROR_PID" 2>/dev/null; then
  echo "http pool pipe: mirror server failed to start" >&2
  exit 1
fi

pipe_pool_keygen "$TMP/bob"
MSG="http-pool e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"
pipe_pool_encrypt "$TMP/bob" "$TMP/msg.txt" "$TMP/msg.wire"

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_url = "http://127.0.0.1:${MIRROR_PORT}"
pool_file = "$POOL_FILE"
cell_size_L = 4096
epoch_interval_ms = 50
sss_k = 2
sss_n = 3
EOF

export ITS_PROD_GATE=1
"$ROUTING" -c "$TMP/pool.toml" client-send --pool --file "$TMP/msg.wire" \
  --ratchet-seed-file "$TMP/ratchet.seed"
"$ROUTING" -c "$TMP/pool.toml" client-receive --pool --continuous \
  --ratchet-seed-file "$TMP/ratchet.seed" --out "$TMP/recv.wire" --timeout-secs 20

if [[ -d "$POOL_FILE" ]]; then
  echo "http pool pipe: file fallback should not be used under ITS_PROD_GATE" >&2
  exit 1
fi

pipe_pool_decrypt "$TMP/bob" "$TMP/recv.wire" "$TMP/out.txt"
[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_http_pool_e2e.sh: OK"
