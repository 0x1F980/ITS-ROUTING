#!/usr/bin/env bash
# Math-only verification gate (M1–M8). No cargo, no E2E pipes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM_ROOT="$(cd "$ROOT/../ITS-asymmetric" 2>/dev/null && pwd || true)"
TL_ROOT="$(cd "$ROOT/../ITS-self_enclosed_timelock/mathematics/stl" 2>/dev/null && pwd || true)"

echo "=== verify_math.sh — Lean certificate only ==="

echo "[1/9] lake build routing-math-cert"
cd "$ROOT/mathematics"
lake update
lake build routing-math-cert

echo "[2/9] grep sorry (ROUTING mathematics)"
if grep -r --include='*.lean' 'sorry' . 2>/dev/null | grep -v '.lake'; then
  echo "FAIL: sorry found in ROUTING mathematics"
  exit 1
fi

if [[ -n "${ASYM_ROOT:-}" && -d "$ASYM_ROOT/mathematics" ]]; then
  echo "[3/9] grep sorry (ITS-asymmetric mathematics)"
  if grep -r --include='*.lean' 'sorry' "$ASYM_ROOT/mathematics" 2>/dev/null | grep -v '.lake'; then
    echo "FAIL: sorry found in ITS-asymmetric mathematics"
    exit 1
  fi
else
  echo "[3/9] skip asymmetric sorry scan (ITS-asymmetric not found)"
fi

echo "[4/9] smoke UnattackableCertificate.lean"
lake env lean UnattackableCertificate.lean

echo "[5/9] smoke Transport/FiniteMutualInfo.lean"
lake env lean Transport/FiniteMutualInfo.lean

OTM_ROOT="$(cd "$ROOT/../ITS-OTM_public_attestation" 2>/dev/null && pwd || true)"
if [[ -n "${OTM_ROOT:-}" && -f "$OTM_ROOT/mathematics/Otm/OtmIntegrity.lean" ]]; then
  echo "[6/9] smoke Otm/OtmIntegrity.lean (ITS-OTM)"
  (cd "$OTM_ROOT/mathematics" && lake env lean Otm/OtmIntegrity.lean)
else
  echo "[6/9] skip OTM smoke (ITS-OTM mathematics not found)"
fi

if [[ -f "$ROOT/mathematics/MasterTheorem.lean" ]]; then
  echo "[7/9] smoke MasterTheorem.lean (M10 Sprint 3)"
  lake env lean MasterTheorem.lean
else
  echo "[7/9] skip MasterTheorem smoke (not found)"
fi

if [[ -n "${TL_ROOT:-}" && -f "$TL_ROOT/Stl/Security/Deniability.lean" ]]; then
  echo "[8/9] smoke Stl/Security/Deniability.lean (M14 C4)"
  (cd "$TL_ROOT" && lake env lean Stl/Security/Deniability.lean)
else
  echo "[8/9] skip timelock Deniability smoke (ITS-timelock stl not found)"
fi

if [[ -f "$ROOT/mathematics/CoercionModel.lean" ]]; then
  echo "[9/9] smoke CoercionModel.lean (M15 coercion)"
  lake env lean CoercionModel.lean
else
  echo "[9/9] skip CoercionModel smoke (not found)"
fi

echo ""
echo "ALL MATH CHECKS PASSED (M1–M8 gate + Sprint 1/2/3 smoke)"
