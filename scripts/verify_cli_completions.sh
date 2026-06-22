#!/usr/bin/env bash
# M27: completions/man drift gate — ghost subcommands, --pool in all shells, constitution PATH.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KM="${ITS_KM_DIR:-/home/user/ITS-KeyManagement}"
FAIL=0

red() { echo "FAIL: $*" >&2; FAIL=1; }
green() { echo "OK: $*"; }

echo "=== M27: ghost subcommand check (completions + man) ==="
for ghost in status-audit verify-path; do
  if grep -rq "$ghost" "$ROOT/completions" 2>/dev/null; then
    red "ghost '$ghost' found in its-routing completions"
  fi
  if grep -q "$ghost" "$ROOT/man/its-routing.1" 2>/dev/null; then
    red "ghost '$ghost' found in man/its-routing.1"
  fi
done
[[ "$FAIL" -eq 0 ]] && green "no ghost subcommands in completions/man"

echo "=== M27: --pool in all four its-routing completion shells ==="
for f in its-routing.bash its-routing.zsh its-routing.fish its-routing.ps1; do
  if grep -q '\-\-pool' "$ROOT/completions/$f"; then
    green "--pool in completions/$f"
  else
    red "--pool missing in completions/$f"
  fi
done

echo "=== M27: --pool-dir in all four its-km completion shells ==="
for f in its-km.bash its-km.zsh its-km.fish its-km.ps1; do
  if grep -q '\-\-pool-dir' "$KM/completions/$f"; then
    green "--pool-dir in $KM/completions/$f"
  else
    red "--pool-dir missing in $KM/completions/$f"
  fi
done

echo "=== M27: constitution binaries on PATH ==="
for bin in its-km its-routing its_asymmetric; do
  if command -v "$bin" >/dev/null 2>&1; then
    green "$bin on PATH ($(command -v "$bin"))"
  else
    # Allow release build paths when not installed globally
    found=0
    for candidate in \
      "$ROOT/target/release/$bin" \
      "$KM/target/release/its-km" \
      "${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}/target/release/its_asymmetric"; do
      if [[ "$bin" == "its_asymmetric" && "$candidate" == *"/its-km" ]]; then
        continue
      fi
      if [[ -x "$candidate" ]]; then
        green "$bin found at $candidate (not on PATH — set PATH for operator use)"
        found=1
        break
      fi
    done
    [[ "$found" -eq 1 ]] || red "$bin not on PATH and no release binary found"
  fi
done

echo "=== M27: print_usage sync spot-check ==="
if [[ -x "$ROOT/target/release/its-routing" ]]; then
  if "$ROOT/target/release/its-routing" --help 2>&1 | grep -q '\-\-pool'; then
    green "its-routing --help mentions --pool"
  else
    red "its-routing --help missing --pool"
  fi
else
  echo "SKIP: its-routing binary not built (run cargo build -p its_routing)"
fi

if [[ "$FAIL" -eq 0 ]]; then
  echo "verify_cli_completions.sh: ALL CHECKS PASSED"
else
  echo "verify_cli_completions.sh: SOME CHECKS FAILED" >&2
  exit 1
fi
