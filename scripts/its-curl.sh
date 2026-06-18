#!/usr/bin/env bash
# its-curl — POST ITS wire with ALPN-style headers (Eco D).
set -euo pipefail

usage() {
  cat <<'EOF'
its-curl — POST ITS .wire to HTTP endpoint (ALPN its-wire/1-compact default)

Usage:
  its-curl.sh URL --pk bob.public.key --file msg.txt
  its-curl.sh URL --pk bob.public.key --file msg.txt --decrypt --sk bob.secret.key
  its-curl.sh --help

Env:
  ITS_ASYMMETRIC_DIR   path to ITS-asymmetric repo (build source)
  ITS_ASYMMETRIC_BIN   override its_asymmetric binary
  ITS_WIRE_PROFILE     compact (default) | standard — logged only; build must match
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

URL=""
PK=""
FILE=""
SK=""
DECRYPT=0
PROFILE="${ITS_WIRE_PROFILE:-compact}"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
TMP="${TMPDIR:-/tmp}/its_curl_$$"
mkdir -p "$TMP"
trap 'rm -rf "$TMP"' EXIT

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pk) PK="$2"; shift 2 ;;
    --file) FILE="$2"; shift 2 ;;
    --sk) SK="$2"; DECRYPT=1; shift 2 ;;
    --decrypt) DECRYPT=1; shift ;;
    --profile) PROFILE="$2"; shift 2 ;;
    http*) URL="$1"; shift ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 1 ;;
  esac
done

[[ -n "$URL" && -n "$PK" && -n "$FILE" ]] || {
  usage >&2
  exit 1
}

ITS="${ITS_ASYMMETRIC_BIN:-}"
if [[ -z "$ITS" || ! -x "$ITS" ]]; then
  ITS="$ASYM/target/release/its_asymmetric"
fi
if [[ ! -x "$ITS" ]]; then
  FEATS="bundle,parallel,std"
  [[ "$PROFILE" == "compact" ]] && FEATS="compact-wire,${FEATS}"
  cargo build --release --manifest-path "$ASYM/Cargo.toml" --bin its_asymmetric --features "$FEATS"
  ITS="$ASYM/target/release/its_asymmetric"
fi

ALPN="its-wire/1-compact"
CT="application/its-wire+1-compact"
[[ "$PROFILE" == "standard" ]] && ALPN="its-wire/1" && CT="application/its-wire+1"

"$ITS" encrypt --pk "$PK" --in "$FILE" --out "$TMP/out.wire"
WIRE="$TMP/out.wire"

curl -fsS -X POST "$URL" \
  -H "Content-Type: ${CT}" \
  -H "ALPN: ${ALPN}" \
  --data-binary "@$WIRE" \
  -o "$TMP/response.bin" 2>/dev/null || echo "its-curl: upstream optional (wire at $WIRE)"

if [[ "$DECRYPT" -eq 1 && -n "$SK" ]]; then
  "$ITS" decrypt --pk "$PK" --sk "$SK" --in "$WIRE" --out -
fi

echo "its-curl: POST $URL profile=$PROFILE ALPN=$ALPN ($(wc -c < "$WIRE" | tr -d ' ') B wire)"
