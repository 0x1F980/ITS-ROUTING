#!/usr/bin/env bash
# Shared helpers for UES Monocell Pool E2E pipe scripts.
set -euo pipefail

pipe_pool_init() {
  local root="$1"
  local tmp_prefix="${2:-its_pool_e2e}"
  ROOT="$root"
  ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
  TMP="${TMPDIR:-/tmp}/${tmp_prefix}_$$"
  POOL="$TMP/pool"
  mkdir -p "$TMP" "$POOL"
  trap 'rm -rf "$TMP"' EXIT

  if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo required" >&2
    exit 1
  fi

  FEATS="${ITS_ASYM_FEATURES:-bundle,parallel,std,compact-wire}"
  cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "$FEATS" --quiet
  cargo build --release --manifest-path "$ROOT/its_routing/Cargo.toml" --quiet
  ITS="$ASYM/target/release/its_asymmetric"
  ROUTING="$ROOT/target/release/its-routing"
}

pipe_pool_keygen() {
  local out_dir="$1"
  "$ITS" keygen --out-dir "$out_dir" 2>/dev/null || "$ITS" keygen --out "$out_dir"
  dd if=/dev/urandom of="$TMP/ratchet.seed" bs=32 count=1 2>/dev/null
}

pipe_pool_encrypt() {
  local pk_dir="$1"
  local plain="$2"
  local wire="$3"
  "$ITS" encrypt --pk "$pk_dir/public.key" --in "$plain" --out "$wire"
}

pipe_pool_decrypt() {
  local key_dir="$1"
  local wire="$2"
  local out="$3"
  "$ITS" decrypt --sk "$key_dir/secret.key" --pk "$key_dir/public.key" \
    --in "$wire" --out "$out"
}

pipe_pool_write_config() {
  local config_path="$1"
  local pool_file="$2"
  cat > "$config_path" <<EOF
[pool]
transport_mode = "pool"
pool_file = "$pool_file"
cell_size_L = 4096
epoch_interval_ms = 100
sss_k = 2
sss_n = 3
fountain_enabled = false
EOF
}
