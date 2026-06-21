#!/usr/bin/env bash
# Primary E2E gate: ITS-asymmetric encrypt → UES Monocell Pool (file) → decrypt.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_pool_e2e_$$"
POOL="$TMP/pool"
mkdir -p "$TMP" "$POOL"
trap 'rm -rf "$TMP"' EXIT

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo required" >&2
  exit 1
fi

echo "== build its_asymmetric =="
FEATS="bundle,parallel,std,compact-wire"
cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "$FEATS"
ITS="$ASYM/target/release/its_asymmetric"

echo "== build its-routing (pool+aeh+otm) =="
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml"
ROUTING="$ROOT/target/release/its-routing"

echo "== transport epoch_cell tests =="
cargo test -p its_transport epoch_cell --quiet --manifest-path "$ROOT/Cargo.toml"

echo "== keygen + encrypt =="
"$ITS" keygen --out-dir "$TMP/bob" 2>/dev/null || "$ITS" keygen --out "$TMP/bob"

MSG="its-pool e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"
"$ITS" encrypt --pk "$TMP/bob/public.key" --in "$TMP/msg.txt" --out "$TMP/msg.wire"

dd if=/dev/urandom of="$TMP/ratchet.seed" bs=32 count=1 2>/dev/null

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$POOL"
cell_size_L = 4096
epoch_interval_ms = 100
sss_k = 2
sss_n = 3
fountain_enabled = false
EOF

echo "== client-send --pool =="
"$ROUTING" -c "$TMP/pool.toml" client-send --pool --file "$TMP/msg.wire" \
  --ratchet-seed-file "$TMP/ratchet.seed"

echo "== client-receive --pool =="
"$ROUTING" -c "$TMP/pool.toml" client-receive --pool \
  --ratchet-seed-file "$TMP/ratchet.seed" --out "$TMP/recv.wire"

if [[ ! -f "$TMP/recv.wire" ]]; then
  echo "receive FAILED: no output at $TMP/recv.wire" >&2
  exit 1
fi

echo "== decrypt received wire =="
"$ITS" decrypt --sk "$TMP/bob/secret.key" --pk "$TMP/bob/public.key" \
  --in "$TMP/recv.wire" --out "$TMP/out.txt"

OUT="$(cat "$TMP/out.txt")"
if [[ "$OUT" != "$MSG" ]]; then
  echo "round-trip FAILED: '$OUT' != '$MSG'" >&2
  exit 1
fi

echo "pipe_its_pool_e2e.sh: OK ($MSG)"
