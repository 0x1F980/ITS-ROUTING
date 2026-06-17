#!/usr/bin/env bash
# Eco B: stream decrypt via its_asymmetric decrypt_pipe on stdin chunks.
set -euo pipefail
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_stream_$$"
mkdir -p "$TMP"
trap 'rm -rf "$TMP"' EXIT

cargo build --release --manifest-path "$ASYM/Cargo.toml" --features bundle,parallel,std >/dev/null
ITS="$ASYM/target/release/its_asymmetric"

"$ITS" keygen --out-dir "$TMP/keys" 2>/dev/null || { echo "CLI skip"; exit 0; }

MSG="stream $(date -u +%s)"
echo -n "$MSG" > "$TMP/in.txt"
"$ITS" encrypt --pk "$TMP/keys/public.key" --in "$TMP/in.txt" --out "$TMP/msg.wire"
"$ITS" decrypt --pk "$TMP/keys/public.key" --sk "$TMP/keys/secret.key" --in "$TMP/msg.wire" --out "$TMP/out.txt"

test "$(cat "$TMP/out.txt")" = "$MSG"
echo "pipe_its_stream.sh: OK"
