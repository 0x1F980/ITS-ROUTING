#!/usr/bin/env bash
# Pool wire E2E: ITS-asymmetric encrypt → UES Monocell Pool → receive → decrypt + integration test.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS="$ROOT/scripts"
# shellcheck source=scripts/lib/pipe_pool_common.sh
source "$SCRIPTS/lib/pipe_pool_common.sh"

pipe_pool_init "$ROOT" "its_routing_e2e"

echo "== transport unit gates =="
cargo test -p its_transport --quiet --manifest-path "$ROOT/Cargo.toml"

echo "== keygen =="
pipe_pool_keygen "$TMP/bob"

MSG="its-routing pool e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"

echo "== wire encrypt =="
pipe_pool_encrypt "$TMP/bob" "$TMP/msg.txt" "$TMP/msg.wire"

pipe_pool_write_config "$TMP/pool.toml" "$POOL"

echo "== client-send --pool =="
"$ROUTING" -c "$TMP/pool.toml" client-send --pool --file "$TMP/msg.wire" \
  --ratchet-seed-file "$TMP/ratchet.seed"

echo "== client-receive --pool =="
"$ROUTING" -c "$TMP/pool.toml" client-receive --pool \
  --ratchet-seed-file "$TMP/ratchet.seed" --out "$TMP/recv.wire" --timeout-secs 25

if [[ ! -f "$TMP/recv.wire" ]]; then
  echo "receive FAILED: no output at $TMP/recv.wire" >&2
  exit 1
fi

echo "== decrypt received wire =="
pipe_pool_decrypt "$TMP/bob" "$TMP/recv.wire" "$TMP/out.txt"

OUT="$(cat "$TMP/out.txt")"
if [[ "$OUT" != "$MSG" ]]; then
  echo "round-trip FAILED: '$OUT' != '$MSG'" >&2
  exit 1
fi

echo "== cargo integration test routing_wire_pool_e2e =="
export ITS_ASYMMETRIC_BIN="$ITS"
cargo test --test routing_wire_pool_e2e -p its_routing --quiet --manifest-path "$ROOT/Cargo.toml"

echo "pipe_its_routing_e2e.sh: OK ($MSG)"
