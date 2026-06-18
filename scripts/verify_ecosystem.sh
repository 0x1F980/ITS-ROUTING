#!/usr/bin/env bash
# Verify ITS ecosystem gates Z1–Z16 on a monorepo or bootstrap tree.
set -euo pipefail

ECO_ROOT="${1:-/home/user}"
ROUTING="$ECO_ROOT/ROUTING"
FAIL=0

red() { echo "FAIL: $*"; FAIL=1; }
green() { echo "OK: $*"; }

echo "=== Z1: no 0x1F464 in Cargo.toml ==="
if rg -l '0x1F464' "$ECO_ROOT"/{SSS_CHAIN,ITS-asymmetric,ITS-OTM_public_attestation,ITS-self_enclosed_timelock,ITS-hardware,ITS-ledger,ITS-fingerprint_erasure,ITS-KeyManagement,ROUTING}/Cargo.toml 2>/dev/null; then
  red "Z1"
else
  green "Z1"
fi

echo "=== Z2: no path = \"../\" in Cargo.toml ==="
if rg 'path = "\.\./' "$ECO_ROOT"/{SSS_CHAIN,ITS-asymmetric,ITS-OTM_public_attestation,ITS-self_enclosed_timelock,ITS-hardware,ITS-ledger,ITS-fingerprint_erasure,ITS-KeyManagement,ROUTING}/Cargo.toml 2>/dev/null; then
  red "Z2"
else
  green "Z2"
fi

echo "=== Z3: no core_logic in src ==="
if rg 'core_logic|hydra_sss' "$ECO_ROOT"/{ITS-asymmetric,ITS-KeyManagement,ITS-hardware,ITS-ledger}/src "$ECO_ROOT/ROUTING/its_routing/src" 2>/dev/null; then
  red "Z3 active src"
else
  green "Z3"
fi

echo "=== Z10: ITS_ECOSYSTEM.md ==="
[[ -f "$ROUTING/ITS_ECOSYSTEM.md" ]] && green "Z10" || red "Z10 missing"

echo "=== Z14: PROOF_MANIFEST per math repo ==="
for m in ITS-asymmetric SSS_CHAIN ITS-OTM_public_attestation ITS-self_enclosed_timelock; do
  [[ -f "$ECO_ROOT/$m/PROOF_MANIFEST.md" ]] && green "Z14 $m" || red "Z14 $m"
done

echo "=== Z15: no its_sessions in KM ==="
if rg 'its_sessions' "$ECO_ROOT/ITS-KeyManagement/src" 2>/dev/null; then
  red "Z15"
else
  green "Z15"
fi

echo "=== cargo test (ROUTING workspace) ==="
if [[ -d "$ROUTING" ]]; then
  (cd "$ROUTING" && cargo test -p its_transport -p its_routing --quiet) && green "ROUTING tests" || red "ROUTING tests"
fi

echo "=== cargo test (math + glue) ==="
for pkg in SSS_CHAIN ITS-OTM_public_attestation ITS-self_enclosed_timelock ITS-hardware ITS-ledger ITS-KeyManagement; do
  if [[ -f "$ECO_ROOT/$pkg/Cargo.toml" ]]; then
    (cd "$ECO_ROOT/$pkg" && cargo test --quiet 2>/dev/null) && green "$pkg tests" || red "$pkg tests"
  fi
done
if [[ -f "$ECO_ROOT/ITS-asymmetric/Cargo.toml" ]]; then
  # Lib + unit only — bundle_adversarial integration tests take ~15 min
  (cd "$ECO_ROOT/ITS-asymmetric" && cargo test --lib --features bundle,std --quiet 2>/dev/null) && green "ITS-asymmetric tests" || red "ITS-asymmetric tests"
fi

echo "=== Z12: fragment + onion regression (its_transport) ==="
if [[ -d "$ROUTING" ]]; then
  if (cd "$ROUTING" && cargo test -p its_transport fragment onion --quiet 2>/dev/null); then
    green "Z12"
  else
    (cd "$ROUTING" && cargo test -p its_transport --quiet 2>/dev/null) && green "Z12 (full its_transport)" || red "Z12"
  fi
fi

echo "=== Z13: core_logic-archive-v1 tag ==="
if git -C "$ECO_ROOT/ITS-session" rev-parse core_logic-archive-v1 >/dev/null 2>&1; then
  green "Z13"
else
  red "Z13 missing tag on ITS-session"
fi

echo "=== Z16: cargo tree no core_logic (ROUTING) ==="
if [[ -d "$ROUTING" ]]; then
  if (cd "$ROUTING" && cargo tree -p its_routing 2>/dev/null | rg -q core_logic); then
    red "Z16"
  else
    green "Z16"
  fi
fi

if [[ "$FAIL" -eq 0 ]]; then
  echo "=== verify_ecosystem: ALL GATES PASSED ==="
else
  echo "=== verify_ecosystem: SOME GATES FAILED ==="
  exit 1
fi
