#!/usr/bin/env bash
# Clone ITS ecosystem repos at tagged releases into ./its-ecosystem/
#
# ECOSYSTEM_TAG defaults to v2.0.0 for UES v2.0 ship gate.
# Pre-UES baseline was v1.1.0; frozen snapshots: ECOSYSTEM_TAG=v1.0.0 ./scripts/bootstrap.sh
set -euo pipefail

ROOT="${1:-./its-ecosystem}"
TAG="${ECOSYSTEM_TAG:-v2.0.0}"
ORG="git@github.com:0x1F980"

mkdir -p "$ROOT"
cd "$ROOT"

clone_repo() {
  local name="$1"
  local branch="${2:-master}"
  if [[ -d "$name" ]]; then
    echo "skip $name (exists)"
    return
  fi
  git clone --branch "$branch" "$ORG/$name.git" "$name" 2>/dev/null || \
    git clone "$ORG/$name.git" "$name"
  if [[ -d "$name/.git" ]]; then
    (cd "$name" && git checkout "$TAG" 2>/dev/null || git checkout "$branch")
  fi
}

clone_repo "SSS_CHAIN"
clone_repo "ITS-asymmetric" main
clone_repo "ITS-OTM_public_attestation" main
clone_repo "ITS-self_enclosed_timelock"
clone_repo "ITS-ROUTING"
clone_repo "ITS-hardware"
clone_repo "ITS-ledger"
clone_repo "ITS-FINGERPRINT_ERASURE"
clone_repo "ITS-KeyManagement" main

echo "Bootstrap complete under $ROOT (tag=$TAG)"
echo "Run: ROUTING/scripts/verify_ecosystem.sh $ROOT"
echo "Run: ROUTING/scripts/pipe_its_routing_e2e.sh"
echo "Run: ROUTING/scripts/pipe_its_proxy_e2e.sh  # its-wire/1 ALPN"
