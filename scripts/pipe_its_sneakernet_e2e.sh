#!/usr/bin/env bash
# Sneakernet / file courier resilience: publish cells, Eve deletes one share epoch, still reconstruct.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="${TMPDIR:-/tmp}/its_sneakernet_e2e_$$"
POOL="$TMP/pool"
mkdir -p "$TMP" "$POOL"
trap 'rm -rf "$TMP"' EXIT

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo required" >&2
  exit 1
fi

cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
ROUTING="$ROOT/target/release/its-routing"

dd if=/dev/urandom of="$TMP/ratchet.seed" bs=32 count=1 2>/dev/null
MSG="sneakernet deletion-resilience test payload"
echo -n "$MSG" > "$TMP/msg.bin"

cat > "$TMP/pool.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$POOL"
cell_size_L = 2048
sss_k = 2
sss_n = 3
EOF

"$ROUTING" -c "$TMP/pool.toml" client-send --pool --file "$TMP/msg.bin" \
  --ratchet-seed-file "$TMP/ratchet.seed"

CELLS=("$POOL"/epoch_*.bin)
if [[ ${#CELLS[@]} -lt 3 ]]; then
  echo "expected at least 3 epoch cells, got ${#CELLS[@]}" >&2
  exit 1
fi

DELETED="${CELLS[0]}"
echo "== Eve deletes one courier file: $(basename "$DELETED") =="
rm -f "$DELETED"

"$ROUTING" -c "$TMP/pool.toml" client-receive --pool \
  --ratchet-seed-file "$TMP/ratchet.seed" --out "$TMP/out.bin"

OUT="$(cat "$TMP/out.bin")"
if [[ "$OUT" != "$MSG" ]]; then
  echo "sneakernet FAILED after deletion: '$OUT' != '$MSG'" >&2
  exit 1
fi

echo "pipe_its_sneakernet_e2e.sh: OK (O_net=empty path, deletion tolerated)"
