#!/usr/bin/env bash
# Math-only verification gate (M1–M16). No cargo, no E2E pipes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM_ROOT="$(cd "$ROOT/../ITS-asymmetric" 2>/dev/null && pwd || true)"
TL_ROOT="$(cd "$ROOT/../ITS-self_enclosed_timelock/mathematics/stl" 2>/dev/null && pwd || true)"
OTM_ROOT="$(cd "$ROOT/../ITS-OTM_public_attestation" 2>/dev/null && pwd || true)"
MATH="$ROOT/mathematics"

echo "=== verify_math.sh — Lean certificate (M1–M16) ==="

echo "[1/12] M1–M8: lake build routing-math-cert"
cd "$MATH"
lake update
lake build routing-math-cert

echo "[2/12] M9: no mutualInfo := 0 stub in ROUTING mathematics"
if grep -r --include='*.lean' -E '^(def|noncomputable def|abbrev)\s+mutualInfo[^:]*:=\s*0\b' "$MATH" 2>/dev/null | grep -v '.lake'; then
  echo "FAIL: mutualInfo := 0 stub found (use Transport/FiniteMutualInfo.lean)"
  exit 1
fi

echo "[3/12] M11: grep sorry (ROUTING mathematics)"
if grep -r --include='*.lean' 'sorry' "$MATH" 2>/dev/null | grep -v '.lake'; then
  echo "FAIL: sorry found in ROUTING mathematics"
  exit 1
fi

if [[ -n "${ASYM_ROOT:-}" && -d "$ASYM_ROOT/mathematics" ]]; then
  echo "[4/12] M11: grep sorry (ITS-asymmetric mathematics)"
  if grep -r --include='*.lean' 'sorry' "$ASYM_ROOT/mathematics" 2>/dev/null | grep -v '.lake'; then
    echo "FAIL: sorry found in ITS-asymmetric mathematics"
    exit 1
  fi
else
  echo "[4/12] M11: skip asymmetric sorry scan (ITS-asymmetric not found)"
fi

echo "[5/12] M7: smoke UnattackableCertificate.lean"
lake env lean UnattackableCertificate.lean

echo "[6/12] M9: smoke Transport/FiniteMutualInfo.lean"
lake env lean Transport/FiniteMutualInfo.lean

if [[ -n "${OTM_ROOT:-}" && -f "$OTM_ROOT/mathematics/Otm/OtmIntegrity.lean" ]]; then
  echo "[7/12] M12: smoke Otm/OtmIntegrity.lean (ITS-OTM C2 import)"
  (cd "$OTM_ROOT/mathematics" && lake env lean Otm/OtmIntegrity.lean)
  if ! grep -q 'import Otm.OtmIntegrity' "$MATH/IntegrityAxiom.lean"; then
    echo "FAIL: IntegrityAxiom.lean must import Otm.OtmIntegrity (M12)"
    exit 1
  fi
else
  echo "[7/12] M12: skip OTM smoke (ITS-OTM mathematics not found)"
fi

if [[ -f "$MATH/MasterTheorem.lean" ]]; then
  echo "[8/13] M10: smoke MasterTheorem.lean"
  lake env lean MasterTheorem.lean
else
  echo "[8/13] M10: skip MasterTheorem smoke (not found)"
fi

if [[ -f "$MATH/MasterTheoremV6.lean" ]]; then
  echo "[9/13] M17: smoke MasterTheoremV6.lean (v6 ecosystem cert)"
  lake env lean MasterTheoremV6.lean
else
  echo "[9/13] M17: skip MasterTheoremV6 smoke (not found)"
fi

if [[ -n "${TL_ROOT:-}" && -f "$TL_ROOT/Stl/Security/Deniability.lean" ]]; then
  echo "[10/13] M14: smoke Stl/Security/Deniability.lean (C4 timelock)"
  (cd "$TL_ROOT" && lake env lean Stl/Security/Deniability.lean)
else
  echo "[10/13] M14: skip timelock Deniability smoke (ITS-timelock stl not found)"
fi

if [[ -f "$MATH/CoercionModel.lean" ]]; then
  echo "[11/13] M15: smoke CoercionModel.lean (coercion model)"
  lake env lean CoercionModel.lean
else
  echo "[11/13] M15: skip CoercionModel smoke (not found)"
fi

echo "[12/13] M13: PROOF_MANIFEST v6 CORE one-liner"
if [[ ! -f "$ROOT/PROOF_MANIFEST.md" ]]; then
  echo "FAIL: PROOF_MANIFEST.md missing (M13)"
  exit 1
fi
if ! grep -q 'v4 MI status' "$ROOT/PROOF_MANIFEST.md"; then
  echo "FAIL: PROOF_MANIFEST.md must include v4 MI status column (M13)"
  exit 1
fi
if ! grep -q 'finite-MI' "$ROOT/PROOF_MANIFEST.md"; then
  echo "FAIL: PROOF_MANIFEST.md must document finite-MI claims (M13)"
  exit 1
fi

echo "[13/14] M16: cert path isolation (no dev-onion imports)"
DEV_IMPORTS=$(
  grep -r --include='*.lean' -l -E 'import.*(MixAnonymity|ChaffIndistinguishability)' "$MATH" 2>/dev/null \
    | grep -v '.lake' || true
)
if [[ -n "$DEV_IMPORTS" ]]; then
  echo "FAIL: dev-onion modules imported in cert path (M16):"
  echo "$DEV_IMPORTS"
  exit 1
fi
if grep -E 'MixAnonymity|ChaffIndistinguishability' "$MATH/lakefile.lean" \
  | grep -q 'routing-math-cert'; then
  echo "FAIL: routing-math-cert must not list dev-onion roots (M16)"
  exit 1
fi

echo "[14/14] M18: no Prop := True stub in ROUTING mathematics"
if grep -r --include='*.lean' 'Prop := True' "$MATH" 2>/dev/null | grep -v '.lake'; then
  echo "FAIL: Prop := True stub found (prove or document Outside)"
  exit 1
fi

echo ""
echo "ALL MATH CHECKS PASSED (M1–M18)"
