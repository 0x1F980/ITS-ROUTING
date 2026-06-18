#!/usr/bin/env bash
# Eco D: its-curl → local its_wire_proxy roundtrip (no external server).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_proxy_e2e_$$"
mkdir -p "$TMP/keys" "$TMP/inbox"
trap 'rm -rf "$TMP"' EXIT

echo "== build its_asymmetric (compact) =="
cargo build --release --manifest-path "$ASYM/Cargo.toml" \
  --bin its_asymmetric --features bundle,compact-wire,parallel,std

ITS="$ASYM/target/release/its_asymmetric"

echo "== keygen =="
"$ITS" keygen --out-dir "$TMP/keys"

echo -n "proxy e2e hello" > "$TMP/msg.txt"

python3 "$ROOT/tools/its_wire_proxy.py" --port 9876 --out-dir "$TMP/inbox" &
PROXY_PID=$!
sleep 0.5
trap 'kill $PROXY_PID 2>/dev/null; rm -rf "$TMP"' EXIT

export ITS_ASYMMETRIC_BIN="$ITS"
export ITS_WIRE_PROFILE=compact
chmod +x "$ROOT/scripts/its-curl.sh"
"$ROOT/scripts/its-curl.sh" "http://127.0.0.1:9876/its/wire" \
  --pk "$TMP/keys/public.key" \
  --file "$TMP/msg.txt" \
  --decrypt --sk "$TMP/keys/secret.key"

kill $PROXY_PID 2>/dev/null || true
echo "pipe_its_proxy_e2e: OK"
