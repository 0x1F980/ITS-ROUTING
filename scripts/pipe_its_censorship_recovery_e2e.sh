#!/usr/bin/env bash
# Censorship recovery: mirror1 blocked → mirror2 + file pool still works.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_censorship_recovery_$$"
POOL="$TMP/pool"
mkdir -p "$TMP" "$POOL"
trap 'kill ${M1:-} ${M2:-} 2>/dev/null; rm -rf "$TMP"' EXIT

cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "bundle,parallel,std,compact-wire" --quiet
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
ITS="$ASYM/target/release/its_asymmetric"
ROUTING="$ROOT/target/release/its-routing"

python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9201 --store-dir "$TMP/mirror1" &
M1=$!
python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9202 --store-dir "$TMP/mirror2" &
M2=$!
sleep 1.5

"$ITS" keygen --out-dir "$TMP/bob" 2>/dev/null || "$ITS" keygen --out "$TMP/bob"
MSG="censorship-recovery e2e"
echo -n "$MSG" > "$TMP/msg.txt"
"$ITS" encrypt --pk "$TMP/bob/public.key" --in "$TMP/msg.txt" --out "$TMP/msg.wire"
dd if=/dev/urandom of="$TMP/ratchet.seed" bs=32 count=1 2>/dev/null

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$POOL"
pool_url = "http://127.0.0.1:9201"
multi_pool_urls = ["http://127.0.0.1:9202"]
cell_size_L = 4096
epoch_interval_ms = 50
sss_k = 2
sss_n = 3
fountain_enabled = true
EOF

"$ROUTING" -c "$TMP/pool.toml" client-send --pool --file "$TMP/msg.wire" \
  --ratchet-seed-file "$TMP/ratchet.seed"

# Block mirror1
kill $M1 2>/dev/null || true
wait $M1 2>/dev/null || true

"$ROUTING" -c "$TMP/pool.toml" client-receive --pool --continuous \
  --ratchet-seed-file "$TMP/ratchet.seed" --out "$TMP/recv.wire" --timeout-secs 25

"$ITS" decrypt --sk "$TMP/bob/secret.key" --pk "$TMP/bob/public.key" \
  --in "$TMP/recv.wire" --out "$TMP/out.txt"
[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_censorship_recovery_e2e.sh: OK"
