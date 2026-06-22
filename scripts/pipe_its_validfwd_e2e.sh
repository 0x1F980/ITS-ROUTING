#!/usr/bin/env bash
# ValidFwd + witness consensus E2E: unit gate + pool harvest from M_valid only.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/pipe_pool_common.sh
source "$ROOT/scripts/lib/pipe_pool_common.sh"

echo "== ITS-A unit tests (ValidFwd + witness consensus) =="
cargo test -p its_routing --lib valid_forward --quiet --manifest-path "$ROOT/Cargo.toml"
cargo test -p its_routing --lib consensus --quiet --manifest-path "$ROOT/Cargo.toml"

pipe_pool_init "$ROOT" "its_validfwd_e2e"

python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9211 --store-dir "$TMP/mirror1" &
M1=$!
python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9212 --store-dir "$TMP/mirror2" &
M2=$!
python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9213 --store-dir "$TMP/mirror3" --evil-omit &
M3=$!
sleep 1.5
for port in 9211 9212 9213; do
  for _ in $(seq 1 30); do
    if curl -sf "http://127.0.0.1:${port}/pool/cells?from=0" >/dev/null 2>&1; then
      break
    fi
    sleep 0.2
  done
done
trap 'kill ${M1:-} ${M2:-} ${M3:-} 2>/dev/null; rm -rf "$TMP"' EXIT

pipe_pool_keygen "$TMP/bob"
MSG="validfwd-witness e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"
pipe_pool_encrypt "$TMP/bob" "$TMP/msg.txt" "$TMP/msg.wire"

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$POOL"
pool_url = "http://127.0.0.1:9211"
multi_pool_urls = ["http://127.0.0.1:9212", "http://127.0.0.1:9213"]
witness_pool_urls = ["http://127.0.0.1:9212"]
consensus_k = 1
valid_fwd_window = 64
cell_size_L = 4096
epoch_interval_ms = 50
sss_k = 2
sss_n = 3
fountain_enabled = true
EOF

"$ROUTING" -c "$TMP/pool.toml" client-send --pool --file "$TMP/msg.wire" \
  --ratchet-seed-file "$TMP/ratchet.seed"

RECV_LOG="$TMP/recv.log"
"$ROUTING" -c "$TMP/pool.toml" client-receive --pool --continuous \
  --ratchet-seed-file "$TMP/ratchet.seed" --out "$TMP/recv.wire" --timeout-secs 25 \
  2>"$RECV_LOG" | tee "$TMP/recv.stdout"

grep -q "de-whitelisted" "$RECV_LOG" || {
  echo "pipe_its_validfwd_e2e: expected ValidFwd de-whitelist log line" >&2
  cat "$RECV_LOG" >&2
  exit 1
}

pipe_pool_decrypt "$TMP/bob" "$TMP/recv.wire" "$TMP/out.txt"
[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_validfwd_e2e.sh: OK (witness harvest + evil mirror de-whitelisted)"
