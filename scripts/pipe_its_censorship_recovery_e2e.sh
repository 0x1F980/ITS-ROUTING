#!/usr/bin/env bash
# Censorship recovery: mirror1 blocked → mirror2 + file pool still works.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/pipe_pool_common.sh
source "$ROOT/scripts/lib/pipe_pool_common.sh"

pipe_pool_init "$ROOT" "its_censorship_recovery"

python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9201 --store-dir "$TMP/mirror1" &
M1=$!
python3 "$ROOT/deploy/pool-mirror/pool_mirror_server.py" --port 9202 --store-dir "$TMP/mirror2" &
M2=$!
sleep 1.5
trap 'kill ${M1:-} ${M2:-} 2>/dev/null; rm -rf "$TMP"' EXIT

pipe_pool_keygen "$TMP/bob"
MSG="censorship-recovery e2e"
echo -n "$MSG" > "$TMP/msg.txt"
pipe_pool_encrypt "$TMP/bob" "$TMP/msg.txt" "$TMP/msg.wire"

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

pipe_pool_decrypt "$TMP/bob" "$TMP/recv.wire" "$TMP/out.txt"
[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_censorship_recovery_e2e.sh: OK"
