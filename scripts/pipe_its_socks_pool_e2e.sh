#!/usr/bin/env bash
# SOCKS pool proxy E2E (M19 v2): Rust its-pool-proxy + Bob bridge + real HTTP roundtrip.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/pipe_pool_common.sh
source "$ROOT/scripts/lib/pipe_pool_common.sh"

pipe_pool_init "$ROOT" "its_socks_pool_e2e"
pipe_pool_keygen "$TMP/bob"
"$ITS" keygen --out-dir "$TMP/alice" 2>/dev/null || "$ITS" keygen --out "$TMP/alice"

HTTP_PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
SOCKS_PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
MARKER="its-socks-m19-v2-$(date +%s)"
SENT_MARKER="$TMP/alice_sent"
REPLIED_MARKER="$TMP/bob_replied"

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$POOL"
cell_size_L = 4096
epoch_interval_ms = 100
sss_k = 2
sss_n = 3
EOF

echo "== build its-pool-proxy =="
cargo build --release --manifest-path "$ROOT/Cargo.toml" -p its_pool_proxy --quiet
PROXY="$ROOT/target/release/its-pool-proxy"

python3 - <<PY &
import http.server
import socketserver
class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(b"$MARKER\n")
    def log_message(self, *a): pass
socketserver.TCPServer(("127.0.0.1", $HTTP_PORT), H).serve_forever()
PY
HTTP_PID=$!
sleep 0.3

bob_bridge_once() {
  for _ in $(seq 1 120); do
    [[ -f "$SENT_MARKER" ]] && break
    sleep 0.1
  done
  [[ -f "$SENT_MARKER" ]] || return 1
  sleep 0.5

  local recv_wire="$TMP/bob_in.wire"
  rm -f "$recv_wire" "$REPLIED_MARKER"
  "$ROUTING" -c "$TMP/pool.toml" client-receive --pool --continuous \
    --timeout-secs 15 -o "$recv_wire" --ratchet-seed-file "$TMP/ratchet.seed"

  "$ITS" decrypt --sk "$TMP/bob/secret.key" --pk "$TMP/bob/public.key" \
    --in "$recv_wire" --out "$TMP/bob_plain.bin"

  curl -sf --max-time 5 -D - "http://127.0.0.1:${HTTP_PORT}/" > "$TMP/bob_resp.bin"

  "$ITS" encrypt --pk "$TMP/alice/public.key" --in "$TMP/bob_resp.bin" --out "$TMP/bob_out.wire"
  rm -f "$POOL"/epoch_*.bin
  "$ROUTING" -c "$TMP/pool.toml" client-send --pool -f "$TMP/bob_out.wire" \
    --ratchet-seed-file "$TMP/ratchet.seed"
  touch "$REPLIED_MARKER"
}

bob_bridge_once &
BOB_PID=$!

export ITS_PROXY_SENT_MARKER="$SENT_MARKER"
export ITS_PROXY_REPLY_MARKER="$REPLIED_MARKER"
"$PROXY" \
  --listen "127.0.0.1:${SOCKS_PORT}" \
  --config "$TMP/pool.toml" \
  --ratchet-seed-file "$TMP/ratchet.seed" \
  --pk "$TMP/bob/public.key" \
  --sk "$TMP/alice/secret.key" \
  --own-pk "$TMP/alice/public.key" \
  --routing "$ROUTING" \
  --asymmetric "$ITS" \
  --receive-timeout-secs 90 \
  --reply-grace-ms 30000 &
PROXY_PID=$!

for _ in $(seq 1 60); do
  if python3 -c "import socket; s=socket.socket(); s.settimeout(0.2); s.connect(('127.0.0.1', ${SOCKS_PORT})); s.close()" 2>/dev/null; then
    break
  fi
  sleep 0.25
done

BODY="$(python3 - <<PY
import socket, struct
s = socket.create_connection(("127.0.0.1", ${SOCKS_PORT}))
s.settimeout(60)
s.sendall(b"\\x05\\x01\\x00")
s.recv(2)
host = b"127.0.0.1"
port = ${HTTP_PORT}
s.sendall(b"\\x05\\x01\\x00\\x03" + bytes([len(host)]) + host + struct.pack("!H", port))
s.recv(10)
s.sendall(b"GET / HTTP/1.1\\r\\nHost: 127.0.0.1\\r\\n\\r\\n")
print(s.recv(65536).decode("utf-8", "replace"))
PY
)" || { kill $PROXY_PID $BOB_PID $HTTP_PID 2>/dev/null; exit 1; }

kill $PROXY_PID $BOB_PID $HTTP_PID 2>/dev/null || true
wait $PROXY_PID 2>/dev/null || true
wait $BOB_PID 2>/dev/null || true

[[ -f "$REPLIED_MARKER" ]] || {
  echo "pipe_its_socks_pool_e2e: Bob bridge did not reply" >&2
  exit 1
}

[[ "$BODY" == *"$MARKER"* ]] || {
  echo "pipe_its_socks_pool_e2e: expected marker in HTTP body, got: $BODY" >&2
  exit 1
}

echo "pipe_its_socks_pool_e2e.sh: OK (M19 v2 HTTP roundtrip)"
