#!/usr/bin/env bash
# Primary E2E gate: ITS-asymmetric encrypt → UES Monocell Pool (file) → decrypt.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/pipe_pool_common.sh
source "$ROOT/scripts/lib/pipe_pool_common.sh"

pipe_pool_init "$ROOT" "its_pool_e2e"

echo "== transport epoch_cell tests =="
cargo test -p its_transport epoch_cell --quiet --manifest-path "$ROOT/Cargo.toml"

echo "== keygen + encrypt =="
pipe_pool_keygen "$TMP/bob"

MSG="its-pool e2e $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo -n "$MSG" > "$TMP/msg.txt"
pipe_pool_encrypt "$TMP/bob" "$TMP/msg.txt" "$TMP/msg.wire"

pipe_pool_write_config "$TMP/pool.toml" "$POOL"

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
pipe_pool_decrypt "$TMP/bob" "$TMP/recv.wire" "$TMP/out.txt"

OUT="$(cat "$TMP/out.txt")"
if [[ "$OUT" != "$MSG" ]]; then
  echo "round-trip FAILED: '$OUT' != '$MSG'" >&2
  exit 1
fi

echo "pipe_its_pool_e2e.sh: OK ($MSG)"
