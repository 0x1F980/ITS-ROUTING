#!/usr/bin/env bash
# KM constitution sneakernet: its-km send/receive --pool-dir, Eve deletes one epoch cell.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
KM="${ITS_KM_DIR:-/home/user/ITS-KeyManagement}"
TMP="${TMPDIR:-/tmp}/its_km_sneakernet_e2e_$$"
ALICE_POOL="$TMP/alice-pool"
BOB_POOL="$TMP/bob-pool"
VAULT="$TMP/test.km.vault"
KEYDIR="$TMP/km-keys"
mkdir -p "$TMP" "$ALICE_POOL" "$BOB_POOL" "$KEYDIR"
trap 'rm -rf "$TMP"' EXIT

export PATH="$ASYM/target/release:$ROOT/target/release:$KM/target/release:$PATH"
export ITS_ASYMMETRIC_BIN="$ASYM/target/release/its_asymmetric"
export ITS_ROUTING_BIN="$ROOT/target/release/its-routing"

cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "bundle,parallel,std,compact-wire" --quiet
cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
cargo build --release --manifest-path "$KM/Cargo.toml" --quiet

ITS="$ITS_ASYMMETRIC_BIN"
KM_BIN="$KM/target/release/its-km"

"$ITS" keygen --out-dir "$TMP/alice" 2>/dev/null || "$ITS" keygen --out "$TMP/alice"
"$ITS" keygen --out-dir "$TMP/bob" 2>/dev/null || "$ITS" keygen --out "$TMP/bob"

dd if=/dev/urandom of="$TMP/shared-ratchet.seed" bs=32 count=1 2>/dev/null

cat > "$TMP/routing.toml" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$TMP/.unused-pool"
cell_size_L = 4096
epoch_interval_ms = 50
sss_k = 2
sss_n = 3
fountain_enabled = true
valid_fwd_window = 64
EOF

"$KM_BIN" --vault "$VAULT" vault init --vault-key-dir "$KEYDIR"
TRUE_SECRET="$KEYDIR/true/secret.key"

"$KM_BIN" --vault "$VAULT" --true-secret "$TRUE_SECRET" entry add \
  --alias bob --public "$TMP/bob/public.key" \
  --routing-config "$TMP/routing.toml" --routing-dest 1 \
  --transport-ratchet-file "$TMP/shared-ratchet.seed"

"$KM_BIN" --vault "$VAULT" --true-secret "$TRUE_SECRET" vault save

MSG="km-sneakernet e2e ok"
echo -n "$MSG" > "$TMP/msg.txt"

"$KM_BIN" --vault "$VAULT" --true-secret "$TRUE_SECRET" send \
  --contact bob --file "$TMP/msg.txt" --work-dir "$TMP/send-bundle" \
  --pool-dir "$ALICE_POOL"

shopt -s nullglob
CELLS=("$ALICE_POOL"/epoch_*.bin)
if [[ ${#CELLS[@]} -lt 3 ]]; then
  echo "expected at least 3 epoch cells in alice pool, got ${#CELLS[@]}" >&2
  exit 1
fi

mkdir -p "$BOB_POOL"
for f in "$ALICE_POOL"/epoch_*.bin; do
  cp -a "$f" "$BOB_POOL/"
done

DELETED="${CELLS[0]}"
echo "== Eve deletes one courier file after handoff: $(basename "$DELETED") =="
rm -f "$BOB_POOL/$(basename "$DELETED")"

VAULT2="$TMP/bob.km.vault"
KEYDIR2="$TMP/bob-km-keys"
mkdir -p "$KEYDIR2"
"$KM_BIN" --vault "$VAULT2" vault init --vault-key-dir "$KEYDIR2"
TRUE_SECRET2="$KEYDIR2/true/secret.key"

"$KM_BIN" --vault "$VAULT2" --true-secret "$TRUE_SECRET2" entry add \
  --alias alice --public "$TMP/bob/public.key" --secret "$TMP/bob/secret.key" \
  --routing-config "$TMP/routing.toml" --routing-dest 1 \
  --transport-ratchet-file "$TMP/shared-ratchet.seed"

"$KM_BIN" --vault "$VAULT2" --true-secret "$TRUE_SECRET2" vault save

"$KM_BIN" --vault "$VAULT2" --true-secret "$TRUE_SECRET2" receive \
  --contact alice --out "$TMP/out.txt" --work-dir "$TMP/recv-bundle" \
  --pool-dir "$BOB_POOL"

[[ "$(cat "$TMP/out.txt")" == "$MSG" ]]
echo "pipe_its_km_sneakernet_e2e.sh: OK (constitution --pool-dir, deletion tolerated)"
