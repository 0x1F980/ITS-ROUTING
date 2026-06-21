#!/usr/bin/env bash
# HTTP pool E2E with ITS_PROD_GATE=1 (no file fallback).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_http_pool_e2e_$$"
POOL_FILE="$TMP/file-fallback-should-not-exist"
mkdir -p "$TMP"
trap 'kill ${MIRROR_PID:-} 2>/dev/null; rm -rf "$TMP"' EXIT

cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "bundle,parallel,std,compact-wire" --quiet
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
ITS="$ASYM/target/release/its_asymmetric"
ROUTING="$ROOT/target/release/its-routing"

MIRROR_PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port "$MIRROR_PORT" --store-dir "$TMP/mirror-store" &
MIRROR_PID=$!
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

"$ITS" keygen --out-dir "$TMP/bob" 2>/dev/null || "$ITS" keygen --out "$TMP/bob"
MSG="http-pool e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"
"$ITS" encrypt --pk "$TMP/bob/public.key" --in "$TMP/msg.txt" --out "$TMP/msg.wire"
dd if=/dev/urandom of="$TMP/ratchet.seed" bs=32 count=1 2>/dev/null

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

"$ITS" decrypt --sk "$TMP/bob/secret.key" --pk "$TMP/bob/public.key" \
  --in "$TMP/recv.wire" --out "$TMP/out.txt"
[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_http_pool_e2e.sh: OK"
