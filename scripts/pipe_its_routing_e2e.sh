#!/usr/bin/env bash
# v5 full: ITS-asymmetric encrypt → its-routing SSS send → receive → decrypt round-trip.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS="$ROOT/scripts"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_routing_e2e_$$"
mkdir -p "$TMP"
trap 'pkill -f "its-routing.*start-node" 2>/dev/null || true; kill $(jobs -p) 2>/dev/null || true; rm -rf "$TMP"' EXIT

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo required" >&2
  exit 1
fi

echo "== build its_asymmetric =="
cargo build --release --manifest-path "$ASYM/Cargo.toml" --no-default-features --features parallel,std
ITS="$ASYM/target/release/its_asymmetric"

echo "== build its-routing =="
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml"
ROUTING="$ROOT/target/release/its-routing"

echo "== transport unit gates =="
cargo test -p its_transport --quiet --manifest-path "$ROOT/Cargo.toml"

echo "== keygen =="
"$ITS" keygen --out-dir "$TMP/bob"

MSG="its-routing e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"

echo "== wire encrypt =="
"$ITS" encrypt --pk "$TMP/bob/public.key" --in "$TMP/msg.txt" --out "$TMP/msg.wire"

echo "== start 3-hop mesh daemons (chaff) =="
pkill -f 'its-routing.*start-node' 2>/dev/null || true
for p in 18401 18402 18403 18404; do
  if command -v fuser >/dev/null 2>&1; then
    fuser -k "${p}/udp" 2>/dev/null || true
  fi
done
sleep 1

echo "== receiver (bob) =="
"$ROUTING" -c "$SCRIPTS/e2e_bob.toml" client-receive --out "$TMP/recv.wire" --timeout-secs 25 &
RECV_PID=$!
sleep 0.5

for cfg in e2e_node1.toml e2e_node2.toml e2e_node3.toml; do
  "$ROUTING" -c "$SCRIPTS/$cfg" start-node &
  sleep 0.3
done
sleep 1

echo "== client-send wire fragments → dest 4 =="
"$ROUTING" -c "$SCRIPTS/e2e_node1.toml" client-send --file "$TMP/msg.wire" --dest 4

wait "$RECV_PID" || true

if [[ ! -f "$TMP/recv.wire" ]]; then
  echo "receive FAILED: no output at $TMP/recv.wire" >&2
  exit 1
fi

echo "== decrypt received wire =="
"$ITS" decrypt --sk "$TMP/bob/secret.key" --pk "$TMP/bob/public.key" --in "$TMP/recv.wire" --out "$TMP/out.txt"

OUT="$(cat "$TMP/out.txt")"
if [[ "$OUT" != "$MSG" ]]; then
  echo "round-trip FAILED: '$OUT' != '$MSG'" >&2
  exit 1
fi

echo "== cargo integration test routing_wire_onion_e2e =="
export ITS_ASYMMETRIC_BIN="$ITS"
cargo test --test routing_wire_onion_e2e -p its_routing --quiet --manifest-path "$ROOT/Cargo.toml"

echo "pipe_its_routing_e2e.sh: OK ($MSG)"
