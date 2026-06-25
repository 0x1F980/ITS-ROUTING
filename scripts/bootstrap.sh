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
  if ! git clone --branch "$branch" "$ORG/$name.git" "$name" 2>/dev/null; then
    if ! git clone --branch main "$ORG/$name.git" "$name" 2>/dev/null; then
      git clone "$ORG/$name.git" "$name"
    fi
  fi
  if [[ -d "$name/.git" ]]; then
    (cd "$name" && git checkout "$TAG" 2>/dev/null \
      || git checkout "$branch" 2>/dev/null \
      || git checkout main 2>/dev/null \
      || git checkout master 2>/dev/null \
      || true)
  fi
}

clone_repo "SSS_CHAIN"
# GitHub repo is ITS-ASSYMETRIC; checkout dir matches verify_ecosystem layout.
if [[ -d ITS-asymmetric ]]; then
  echo "skip ITS-asymmetric (exists)"
elif git clone --branch main "$ORG/ITS-ASSYMETRIC.git" ITS-asymmetric 2>/dev/null \
  || git clone --branch main "$ORG/ITS-asymmetric.git" ITS-asymmetric 2>/dev/null \
  || git clone "$ORG/ITS-ASSYMETRIC.git" ITS-asymmetric 2>/dev/null \
  || git clone "$ORG/ITS-asymmetric.git" ITS-asymmetric; then
  (cd ITS-asymmetric && git checkout "$TAG" 2>/dev/null \
    || git checkout main 2>/dev/null \
    || git checkout master 2>/dev/null \
    || true)
else
  echo "bootstrap: failed to clone ITS-asymmetric / ITS-ASSYMETRIC" >&2
  exit 1
fi
clone_repo "ITS-OTM_public_attestation" main
clone_repo "ITS-self_enclosed_timelock"
clone_repo "ITS-ROUTING"
clone_repo "sidechannel_resistant_hardware"
clone_repo "ITS-ledger"
clone_repo "ITS-FINGERPRINT_ERASURE"
clone_repo "ITS-KeyManagement" main

# verify_ecosystem.sh expects monorepo-style directory names.
[[ -d ITS-ROUTING && ! -e ROUTING ]] && ln -sfn ITS-ROUTING ROUTING
[[ -d ITS-FINGERPRINT_ERASURE && ! -e ITS-fingerprint_erasure ]] \
  && ln -sfn ITS-FINGERPRINT_ERASURE ITS-fingerprint_erasure

echo "Bootstrap complete under $ROOT (tag=$TAG)"
echo "Run: ROUTING/scripts/verify_ecosystem.sh $ROOT"
echo "Run: ROUTING/scripts/pipe_its_pool_e2e.sh"
echo "Run: ROUTING/scripts/pipe_its_proxy_e2e.sh  # its-wire/1 ALPN"
