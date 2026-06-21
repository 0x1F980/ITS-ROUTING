#!/usr/bin/env bash
# SOCKS pool proxy smoke test (v1.8).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_socks_pool_e2e_$$"
POOL="$TMP/pool"
mkdir -p "$TMP" "$POOL"
trap 'rm -rf "$TMP"' EXIT

cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "bundle,parallel,std,compact-wire" --quiet
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
ITS="$ASYM/target/release/its_asymmetric"
ROUTING="$ROOT/target/release/its-routing"

"$ITS" keygen --out-dir "$TMP/bob" 2>/dev/null || "$ITS" keygen --out "$TMP/bob"
dd if=/dev/urandom of="$TMP/ratchet.seed" bs=32 count=1 2>/dev/null

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$POOL"
cell_size_L = 4096
epoch_interval_ms = 50
sss_k = 2
sss_n = 3
EOF

# Proxy runs full send+receive loop internally for smoke test
timeout 30 python3 "$ROOT/tools/its_pool_proxy.py" \
  --listen 127.0.0.1:19880 \
  --config "$TMP/pool.toml" \
  --ratchet-seed-file "$TMP/ratchet.seed" \
  --pk "$TMP/bob/public.key" \
  --routing "$ROUTING" \
  --asymmetric "$ITS" &
PROXY_PID=$!
sleep 1

python3 - <<'PY' || { kill $PROXY_PID 2>/dev/null; exit 1; }
import socket
s = socket.socket()
s.settimeout(5)
s.connect(("127.0.0.1", 19880))
s.sendall(b"\x05\x01\x00")
assert s.recv(2) == b"\x05\x00"
req = b"\x05\x01\x00\x03" + bytes([9]) + b"example.com" + (80).to_bytes(2, "big")
s.sendall(req)
resp = s.recv(32)
assert len(resp) >= 2
print("socks handshake ok")
PY

kill $PROXY_PID 2>/dev/null || true
echo "pipe_its_socks_pool_e2e.sh: OK"
