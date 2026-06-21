#!/usr/bin/env bash
# Math-only verification gate (M1–M8). No cargo, no E2E pipes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM_ROOT="$(cd "$ROOT/../ITS-asymmetric" 2>/dev/null && pwd || true)"

echo "=== verify_math.sh — Lean certificate only ==="

echo "[1/4] lake build ROUTING/mathematics"
cd "$ROOT/mathematics"
lake build

echo "[2/4] grep sorry (ROUTING mathematics)"
if grep -r --include='*.lean' 'sorry' . 2>/dev/null | grep -v '.lake'; then
  echo "FAIL: sorry found in ROUTING mathematics"
  exit 1
fi

if [[ -n "${ASYM_ROOT:-}" && -d "$ASYM_ROOT/mathematics" ]]; then
  echo "[3/4] grep sorry (ITS-asymmetric mathematics)"
  if grep -r --include='*.lean' 'sorry' "$ASYM_ROOT/mathematics" 2>/dev/null | grep -v '.lake'; then
    echo "FAIL: sorry found in ITS-asymmetric mathematics"
    exit 1
  fi
else
  echo "[3/4] skip asymmetric sorry scan (ITS-asymmetric not found)"
fi

echo "[4/4] smoke UnattackableCertificate.lean"
lake env lean UnattackableCertificate.lean

echo ""
echo "ALL MATH CHECKS PASSED (M1–M8 gate)"
