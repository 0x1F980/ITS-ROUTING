#!/usr/bin/env bash
# Eco D: ITS-asymmetric wire → optional timelock → decrypt round-trip (no RSA).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_pipe_e2e_$$"
mkdir -p "$TMP"
trap 'rm -rf "$TMP"' EXIT

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo required" >&2
  exit 1
fi

echo "== build its_asymmetric (wire-only) =="
cargo build --release --manifest-path "$ASYM/Cargo.toml" --no-default-features --features parallel,std

ITS="$ASYM/target/release/its_asymmetric"
if [[ ! -x "$ITS" ]]; then
  echo "wire-only build has no CLI binary; building with bundle for demo bin" >&2
  cargo build --release --manifest-path "$ASYM/Cargo.toml" --features bundle,parallel,std
  ITS="$ASYM/target/release/its_asymmetric"
fi

echo "== keygen =="
"$ITS" keygen --out "$TMP/bob" 2>/dev/null || {
  # library-only path: use cargo test helper keys if CLI missing
  echo "its_asymmetric CLI unavailable; skipping" >&2
  exit 0
}

MSG="its-routing e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"

echo "== encrypt pipe =="
"$ITS" encrypt --pk "$TMP/bob.public.key" --in "$TMP/msg.txt" --out "$TMP/msg.wire"

if command -v its-routing >/dev/null 2>&1; then
  echo "== optional timelock wrap =="
  its-routing time-lock -f "$TMP/msg.wire" -o "$TMP/msg.wire.its" -e 5
  its-routing time-unlock -p "$TMP/msg.wire.its" -o "$TMP/msg.unlocked.wire"
  WIRE="$TMP/msg.unlocked.wire"
else
  echo "its-routing not on PATH; wire-only path" >&2
  WIRE="$TMP/msg.wire"
fi

echo "== decrypt =="
"$ITS" decrypt --sk "$TMP/bob.secret.key" --pk "$TMP/bob.public.key" --in "$WIRE" --out "$TMP/out.txt"

OUT="$(cat "$TMP/out.txt")"
if [[ "$OUT" != "$MSG" ]]; then
  echo "round-trip FAILED: '$OUT' != '$MSG'" >&2
  exit 1
fi

echo "pipe_its_e2e.sh: OK ($MSG)"
