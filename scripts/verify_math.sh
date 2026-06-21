#!/usr/bin/env bash
# Math-only verification gate (M1–M8). No cargo, no E2E pipes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM_ROOT="$(cd "$ROOT/../ITS-asymmetric" 2>/dev/null && pwd || true)"

echo "=== verify_math.sh — Lean certificate only ==="

echo "[1/6] lake build routing-math-cert"
cd "$ROOT/mathematics"
lake build routing-math-cert

echo "[2/6] grep sorry (ROUTING mathematics)"
if grep -r --include='*.lean' 'sorry' . 2>/dev/null | grep -v '.lake'; then
  echo "FAIL: sorry found in ROUTING mathematics"
  exit 1
fi

if [[ -n "${ASYM_ROOT:-}" && -d "$ASYM_ROOT/mathematics" ]]; then
  echo "[3/6] grep sorry (ITS-asymmetric mathematics)"
  if grep -r --include='*.lean' 'sorry' "$ASYM_ROOT/mathematics" 2>/dev/null | grep -v '.lake'; then
    echo "FAIL: sorry found in ITS-asymmetric mathematics"
    exit 1
  fi
else
  echo "[3/6] skip asymmetric sorry scan (ITS-asymmetric not found)"
fi

echo "[4/6] smoke UnattackableCertificate.lean"
lake env lean UnattackableCertificate.lean

echo "[5/6] smoke Transport/FiniteMutualInfo.lean"
lake env lean Transport/FiniteMutualInfo.lean

OTM_ROOT="$(cd "$ROOT/../ITS-OTM_public_attestation" 2>/dev/null && pwd || true)"
if [[ -n "${OTM_ROOT:-}" && -f "$OTM_ROOT/mathematics/Otm/OtmIntegrity.lean" ]]; then
  echo "[6/6] smoke Otm/OtmIntegrity.lean (ITS-OTM)"
  (cd "$OTM_ROOT/mathematics" && lake env lean Otm/OtmIntegrity.lean)
else
  echo "[6/6] skip OTM smoke (ITS-OTM mathematics not found)"
fi

echo ""
echo "ALL MATH CHECKS PASSED (M1–M8 gate + Sprint 1 smoke)"
