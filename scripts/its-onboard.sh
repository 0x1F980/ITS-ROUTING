#!/usr/bin/env bash
# ITS onboard wizard — vault → routing.toml → contact → test send checklist.
#
# Constitution path only (its-km). Does not invoke raw its-routing client-send.
#
# Usage:
#   ./scripts/its-onboard.sh
#   ITS_TRUE_SECRET=~/.its/km-vault-keys/true/secret.key ./scripts/its-onboard.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROUTING="$(cd "$SCRIPT_DIR/.." && pwd)"
ECO="$(cd "$ROUTING/.." && pwd)"
ITS_HOME="${ITS_HOME:-$HOME/.its}"
VAULT_KEYS="${ITS_VAULT_KEY_DIR:-$ITS_HOME/km-vault-keys}"
TRUE_SECRET="${ITS_TRUE_SECRET:-$VAULT_KEYS/true/secret.key}"
ROUTING_TOML="${ITS_ROUTING_CONFIG:-$ITS_HOME/routing.toml}"
WORK_DIR="${ITS_WORK_DIR:-$ITS_HOME/work}"

need_bin() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "its-onboard: $1 not on PATH — run scripts/its-operator-install.sh first" >&2
    exit 1
  }
}

need_bin its-km

step() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  $*"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

prompt_yes() {
  local msg="$1"
  local reply
  read -r -p "$msg [y/N] " reply || reply=""
  [[ "$reply" =~ ^[Yy]$ ]]
}

mkdir -p "$ITS_HOME" "$WORK_DIR"

step "1/5 — Vault init"
if [[ -f "$TRUE_SECRET" ]]; then
  echo "Found existing vault secret: $TRUE_SECRET"
else
  echo "No vault at $TRUE_SECRET"
  echo ""
  echo "Run vault init (creates key hierarchy under $VAULT_KEYS):"
  echo ""
  echo "  its-km vault init --vault-key-dir $VAULT_KEYS"
  echo ""
  if prompt_yes "Run vault init now?"; then
    its-km vault init --vault-key-dir "$VAULT_KEYS"
  else
    echo "Complete vault init, then re-run this wizard."
    exit 0
  fi
fi

if [[ ! -f "$TRUE_SECRET" ]]; then
  echo "its-onboard: vault secret still missing at $TRUE_SECRET" >&2
  exit 1
fi

step "2/5 — Routing profile (~/.its/routing.toml)"
PROD_TEMPLATE="$ROUTING/config.prod.toml"
if [[ ! -f "$PROD_TEMPLATE" ]]; then
  echo "its-onboard: missing template $PROD_TEMPLATE" >&2
  exit 1
fi

if [[ -f "$ROUTING_TOML" ]]; then
  echo "Using existing: $ROUTING_TOML"
else
  cp "$PROD_TEMPLATE" "$ROUTING_TOML"
  echo "Copied $PROD_TEMPLATE → $ROUTING_TOML"
fi

echo ""
echo "Edit mirror URLs before first online send (REPLACE placeholders in config.prod.toml):"
echo "  • multi_pool_urls   — public UES pool mirrors (≥2 for ITS-A)"
echo "  • witness_pool_urls — independent witnesses (≥3; consensus_k = 2)"
echo ""
echo "  \$EDITOR $ROUTING_TOML"
echo ""
echo "Deploy reference: $ROUTING/deploy/pool-mirror/README.md"
echo "Local smoke:      docker compose -f $ROUTING/deploy/docker/docker-compose.yml up pool-mirror"
echo ""
echo "Offline / USB instead? Copy config.offline.toml:"
echo "  cp $ROUTING/config.offline.toml $ROUTING_TOML"

step "3/5 — Add first contact (entry add)"
echo ""
echo "Exchange public keys OOB, then add the peer:"
echo ""
cat <<EOF
  its-km --true-secret $TRUE_SECRET entry add \\
    --alias CONTACT_ALIAS \\
    --public /path/to/contact.public.key \\
    --routing-config $ROUTING_TOML
EOF
echo ""
echo "entry add auto-copies routing.toml when missing and generates per-contact transport ratchet."
echo ""
echo "Sync transport ratchet OOB (required before first send):"
echo ""
cat <<EOF
  its-km --true-secret $TRUE_SECRET export-qr --contact CONTACT_ALIAS --layer transport-ratchet
  # peer:
  its-km --true-secret PEER_SECRET import-qr --alias YOUR_ALIAS --layer transport-ratchet --payload 'its-km:qr:...'
EOF

step "4/5 — Test send checklist"
echo ""
echo "Before first message, confirm:"
echo "  [ ] its-km, its-routing, its_asymmetric on PATH (its-operator-install.sh)"
echo "  [ ] routing.toml mirror URLs point at live mirrors (not empty, not localhost-only for prod)"
echo "  [ ] contact entry exists with synced transport ratchet"
echo "  [ ] peer has reciprocal entry for you"
echo ""
echo "Alice send:"
echo "  its-km --true-secret $TRUE_SECRET send --contact CONTACT_ALIAS --file /path/to/doc.pdf"
echo ""
echo "Bob receive:"
echo "  its-km --true-secret BOB_SECRET receive --contact ALICE_ALIAS --out received.pdf"
echo ""
echo "Optional continuous receive (Bob ingress):"
echo "  its-km --true-secret BOB_SECRET receive --contact ALICE_ALIAS --continuous --out-dir $WORK_DIR/inbox"

step "5/5 — Verify"
echo ""
echo "  $ROUTING/scripts/verify_cli_completions.sh"
echo "  $ROUTING/scripts/verify_ecosystem.sh $ECO"
echo ""
echo "Latency profiles (see QUICKSTART §5):"
echo "  config.fast.toml   — lab / LAN (epoch_interval_ms = 50, consensus_k = 1)"
echo "  config.robust.toml — production ITS-A + fountain defaults"
echo ""
echo "its-onboard: wizard complete. Constitution reference: $ROUTING/ITS_CONSTITUTION_CLI.md"
