#!/usr/bin/env bash
# CoverTransport E2E: pool + entropy_sources cover harvest every epoch.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_cover_e2e_$$"
POOL="$TMP/pool"
COVER_LOG="$TMP/cover.log"
mkdir -p "$TMP" "$POOL"
trap 'rm -rf "$TMP"' EXIT

cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "bundle,parallel,std,compact-wire" --quiet
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
ITS="$ASYM/target/release/its_asymmetric"
ROUTING="$ROOT/target/release/its-routing"

"$ITS" keygen --out-dir "$TMP/bob" 2>/dev/null || "$ITS" keygen --out "$TMP/bob"
MSG="cover-harvest e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"
"$ITS" encrypt --pk "$TMP/bob/public.key" --in "$TMP/msg.txt" --out "$TMP/msg.wire"
dd if=/dev/urandom of="$TMP/ratchet.seed" bs=32 count=1 2>/dev/null

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

"$ITS" decrypt --sk "$TMP/bob/secret.key" --pk "$TMP/bob/public.key" \
  --in "$TMP/recv.wire" --out "$TMP/out.txt"
[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_cover_harvest_e2e.sh: OK"
