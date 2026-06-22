#!/usr/bin/env bash
# M28b: prod-like base + --pool-dir must stay file-only — no HTTP mirror or cover harvest.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
KM="${ITS_KM_DIR:-/home/user/ITS-KeyManagement}"
TMP="${TMPDIR:-/tmp}/its_km_pooldir_hazard_$$"
POOL="$TMP/usb-pool"
VAULT="$TMP/test.km.vault"
KEYDIR="$TMP/km-keys"
TRAP_PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
mkdir -p "$TMP" "$POOL" "$KEYDIR"
trap 'kill ${TRAP_PID:-} 2>/dev/null || true; rm -rf "$TMP"' EXIT

export PATH="$ASYM/target/release:$ROOT/target/release:$KM/target/release:$PATH"
export ITS_ASYMMETRIC_BIN="$ASYM/target/release/its_asymmetric"
export ITS_ROUTING_BIN="$ROOT/target/release/its-routing"

cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "bundle,parallel,std,compact-wire" --quiet
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
cargo build --release --manifest-path "$KM/Cargo.toml" --quiet

ITS="$ITS_ASYMMETRIC_BIN"
KM_BIN="$KM/target/release/its-km"

# Trap server: any connection means HTTP leak (mirror or AEH cover).
python3 - "$TRAP_PORT" "$TMP/trap.log" <<'PY' &
import socket, sys
port, log = int(sys.argv[1]), sys.argv[2]
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", port))
s.listen(5)
s.settimeout(0.5)
open(log, "w").write("armed\n")
while True:
    try:
        conn, _ = s.accept()
        conn.close()
        open(log, "a").write("HIT\n")
        sys.exit(42)
    except socket.timeout:
        if open(log).read().strip() == "stop":
            break
PY
TRAP_PID=$!
for _ in $(seq 1 50); do
  [[ -f "$TMP/trap.log" ]] && break
  sleep 0.05
done

"$ITS" keygen --out-dir "$TMP/alice" 2>/dev/null || "$ITS" keygen --out "$TMP/alice"
"$ITS" keygen --out-dir "$TMP/bob" 2>/dev/null || "$ITS" keygen --out "$TMP/bob"
dd if=/dev/urandom of="$TMP/shared-ratchet.seed" bs=32 count=1 2>/dev/null

# Prod-like config: mirrors + BlockCypher AEH — must be neutralized by --pool-dir override.
cat > "$TMP/prod-like.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = ".its-pool"
cell_size_L = 4096
epoch_interval_ms = 50
sss_k = 2
sss_n = 3
fountain_enabled = true
valid_fwd_window = 64
consensus_k = 2
multi_pool_urls = [
  "http://127.0.0.1:${TRAP_PORT}",
]
witness_pool_urls = [
  "http://127.0.0.1:${TRAP_PORT}",
]

[aeh]
entropy_sources = [
  "http://127.0.0.1:${TRAP_PORT}/aeh",
]
EOF

"$KM_BIN" --vault "$VAULT" vault init --vault-key-dir "$KEYDIR"
TRUE_SECRET="$KEYDIR/true/secret.key"

"$KM_BIN" --vault "$VAULT" --true-secret "$TRUE_SECRET" entry add \
  --alias bob --public "$TMP/bob/public.key" \
  --routing-config "$TMP/prod-like.toml" --routing-dest 1 \
  --transport-ratchet-file "$TMP/shared-ratchet.seed"
"$KM_BIN" --vault "$VAULT" --true-secret "$TRUE_SECRET" vault save

echo -n "hazard-test" > "$TMP/msg.txt"
WORK="$TMP/send-work"
"$KM_BIN" --vault "$VAULT" --true-secret "$TRUE_SECRET" send \
  --contact bob --file "$TMP/msg.txt" --work-dir "$WORK" \
  --pool-dir "$POOL"

OVERRIDE="$WORK/routing.override.toml"
[[ -f "$OVERRIDE" ]] || { echo "missing routing.override.toml" >&2; exit 1; }
grep -q "multi_pool_urls = \[\]" "$OVERRIDE" || { echo "override did not clear multi_pool_urls" >&2; exit 1; }
grep -q "witness_pool_urls = \[\]" "$OVERRIDE" || { echo "override did not clear witness_pool_urls" >&2; exit 1; }
grep -q "entropy_sources = \[\]" "$OVERRIDE" || { echo "override did not clear entropy_sources" >&2; exit 1; }
grep -q "pool_file = \"$POOL\"" "$OVERRIDE" || { echo "override pool_file mismatch" >&2; exit 1; }
if grep -q "127.0.0.1:${TRAP_PORT}" "$OVERRIDE"; then
  echo "trap URL still in override" >&2
  exit 1
fi

CELLS=("$POOL"/epoch_*.bin)
[[ ${#CELLS[@]} -ge 1 ]] || { echo "expected epoch cells in pool dir" >&2; exit 1; }

echo stop > "$TMP/trap.log"
wait "$TRAP_PID" 2>/dev/null || true
if grep -q HIT "$TMP/trap.log" 2>/dev/null; then
  echo "HTTP trap server was contacted — prod+pool-dir hazard not mitigated" >&2
  exit 1
fi

echo "pipe_its_km_pooldir_prod_hazard.sh: OK (file-only override, no HTTP)"
exit 0
