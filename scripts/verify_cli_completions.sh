#!/usr/bin/env bash
# M27 v2: completions/man drift gate — ghost subcommands, cli.rs sync, constitution PATH.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ECO_ROOT="$(cd "$ROOT/.." && pwd)"
KM="${ITS_KM_DIR:-$ECO_ROOT/ITS-KeyManagement}"
ASY="${ITS_ASYMMETRIC_DIR:-$ECO_ROOT/ITS-asymmetric}"
FAIL=0

export ITS_KM_DIR="$KM"
export ITS_ASYMMETRIC_DIR="$ASY"

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
  if [[ -f "$KM/completions/$f" ]] && grep -q '\-\-pool-dir' "$KM/completions/$f"; then
    green "--pool-dir in $KM/completions/$f"
  else
    red "--pool-dir missing in $KM/completions/$f"
  fi
done

echo "=== M27: its_asymmetric completions in all four shells ==="
for f in its_asymmetric.bash its_asymmetric.zsh its_asymmetric.fish its_asymmetric.ps1; do
  if [[ -f "$ASY/completions/$f" ]]; then
    if grep -qE 'keygen|encrypt' "$ASY/completions/$f"; then
      green "constitution commands in $ASY/completions/$f"
    else
      red "its_asymmetric completions missing key commands in $f"
    fi
  else
    red "its_asymmetric completion missing: $ASY/completions/$f"
  fi
done

echo "=== M27: its_asymmetric bash subcommands vs cli.rs (constitution subset) ==="
ASY_CLI="$ASY/src/bin/its_asymmetric.rs"
constitution_asym="bundle-append bundle-open decrypt decrypt-file encrypt encrypt-file fingerprint keygen verify verify-file"
if [[ -f "$ASY_CLI" && -f "$ASY/completions/its_asymmetric.bash" ]]; then
  bash_asym="$(grep -oE 'compgen -W "[^"]+"' "$ASY/completions/its_asymmetric.bash" | head -1 \
    | sed 's/compgen -W "//;s/"$//' | tr ' ' '\n' | grep -v '^help$' | grep -v '^-' | sort -u)"
  for sub in $constitution_asym; do
    echo "$bash_asym" | grep -qx "$sub" || red "its_asymmetric bash completion missing constitution command '$sub'"
  done
  [[ "$FAIL" -eq 0 ]] && green "its_asymmetric constitution commands present in bash completion"
else
  echo "SKIP: its_asymmetric cli or bash completion not found"
fi

echo "=== M27: mailbox-fingerprint only on client-receive (not client-send) ==="
for f in its-routing.bash its-routing.zsh its-routing.fish its-routing.ps1; do
  path="$ROOT/completions/$f"
  if grep -q 'client-send' "$path" && grep -q 'mailbox-fingerprint' "$path"; then
    case "$f" in
      *.bash)
        send_line="$(awk '/client-send\)/,/;;/' "$path" | tr '\n' ' ')"
        recv_line="$(awk '/client-receive\)/,/;;/' "$path" | tr '\n' ' ')"
        if [[ "$send_line" == *mailbox-fingerprint* ]]; then
          red "mailbox-fingerprint on client-send in $f"
        else
          green "client-send clean in $f"
        fi
        [[ "$recv_line" == *mailbox-fingerprint* ]] || red "mailbox-fingerprint missing on client-receive in $f"
        ;;
      *.fish)
        if grep -q '__fish_seen_subcommand_from client-send.*mailbox-fingerprint' "$path" \
          || grep -A1 'client-send' "$path" | grep -q 'mailbox-fingerprint'; then
          if grep 'client-send' "$path" | grep -q 'mailbox-fingerprint'; then
            red "mailbox-fingerprint on client-send in $f"
          fi
        fi
        grep -q 'client-receive.*mailbox-fingerprint\|client-receive".*mailbox-fingerprint' "$path" \
          || grep -q '__fish_seen_subcommand_from client-receive' "$path"
        if grep -q '__fish_seen_subcommand_from client-receive' "$path" \
          && grep -q 'mailbox-fingerprint' "$path"; then
          green "client-receive has mailbox-fingerprint in $f"
        else
          red "mailbox-fingerprint missing on client-receive in $f"
        fi
        ;;
      *)
        if grep -q 'mailbox-fingerprint' "$path"; then
          green "spot-check $f (manual review for send vs receive blocks)"
        fi
        ;;
    esac
  fi
done

echo "=== M27: cli.rs subcommand set vs bash completion ==="
CLI_RS="$ROOT/its_routing/src/cli.rs"
expected_subs="$(grep -E '^\s+"[a-z0-9-]+" =>' "$CLI_RS" | sed -E 's/.*"([^"]+)".*/\1/' | sort -u)"
bash_subs="$(grep -oE 'opts="[^"]+"' "$ROOT/completions/its-routing.bash" | head -1 \
  | sed 's/opts="//;s/"$//' | tr ' ' '\n' | grep -v '^--' | grep -v '^-' | sort -u)"
while IFS= read -r sub; do
  [[ -z "$sub" ]] && continue
  echo "$bash_subs" | grep -qx "$sub" || red "cli.rs subcommand '$sub' missing from bash opts"
done <<< "$expected_subs"
while IFS= read -r sub; do
  [[ -z "$sub" ]] && continue
  echo "$expected_subs" | grep -qx "$sub" || red "bash completion subcommand '$sub' not in cli.rs"
done <<< "$bash_subs"
[[ "$FAIL" -eq 0 ]] && green "cli.rs ↔ bash subcommand set aligned"

echo "=== M27: constitution binaries on PATH ==="
for bin in its-km its-routing its_asymmetric; do
  if command -v "$bin" >/dev/null 2>&1; then
    green "$bin on PATH ($(command -v "$bin"))"
  else
    found=0
    for candidate in \
      "$ROOT/target/release/$bin" \
      "$KM/target/release/its-km" \
      "$ASY/target/release/its_asymmetric"; do
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

echo "=== M27: man pages for constitution binaries ==="
for spec in "$ROOT/man/its-routing.1" "$KM/man/its-km.1" "$ASY/man/its_asymmetric.1"; do
  [[ -f "$spec" ]] && green "man present: $spec" || red "man missing: $spec"
done

echo "=== M27: man default config path (its-routing.1 vs cli.rs) ==="
if grep -q '/etc/its-routing/config.toml' "$ROOT/man/its-routing.1" 2>/dev/null; then
  red "man still claims wrong default /etc/its-routing/config.toml"
else
  green "man default config path matches cli.rs (config.toml cwd fallback)"
fi

echo "=== M27: its-routing man subcommands ==="
for sub in client-send client-receive start-node time-lock time-unlock time-deny fingerprint-erasure; do
  man_token="${sub//-/\\-}"
  if grep -qF "$man_token" "$ROOT/man/its-routing.1" 2>/dev/null; then
    green "man documents $sub"
  else
    red "man missing subcommand $sub"
  fi
done
for sub in client-export-share client-import-share; do
  man_token="${sub//-/\\-}"
  if grep -qF "$man_token" "$ROOT/man/its-routing.1" 2>/dev/null \
    && grep -qi 'hardware' "$ROOT/man/its-routing.1"; then
    green "man documents hardware-gated $sub"
  else
    red "man missing or unmarked hardware-gated $sub"
  fi
done

echo "=== M27: print_usage sync spot-check ==="
if [[ -x "$ROOT/target/release/its-routing" ]]; then
  help_out="$("$ROOT/target/release/its-routing" --help 2>&1)"
  echo "$help_out" | grep -q '\-\-pool' && green "its-routing --help mentions --pool" \
    || red "its-routing --help missing --pool"
  echo "$help_out" | grep -q 'client-export-share' && green "its-routing --help lists client-export-share" \
    || red "its-routing --help missing client-export-share"
else
  echo "SKIP: its-routing binary not built (run cargo build -p its_routing)"
fi

if [[ -x "$KM/target/release/its-km" ]]; then
  km_help="$("$KM/target/release/its-km" --help 2>&1 || true)"
  echo "$km_help" | grep -q '\-\-pool-dir' && green "its-km --help mentions --pool-dir" \
    || red "its-km --help missing --pool-dir"
else
  echo "SKIP: its-km binary not built under ITS_KM_DIR"
fi

if [[ -x "$ASY/target/release/its_asymmetric" ]]; then
  asym_help="$("$ASY/target/release/its_asymmetric" --help 2>&1 || true)"
  echo "$asym_help" | grep -q 'keygen' && green "its_asymmetric --help lists keygen" \
    || red "its_asymmetric --help missing keygen"
  echo "$asym_help" | grep -q 'encrypt-file' && green "its_asymmetric --help lists encrypt-file" \
    || red "its_asymmetric --help missing encrypt-file"
else
  echo "SKIP: its_asymmetric binary not built under ITS_ASYMMETRIC_DIR"
fi

echo "=== M27: its_asymmetric man subcommands ==="
for sub in keygen encrypt decrypt verify fingerprint encrypt-file decrypt-file; do
  if grep -qF "$sub" "$ASY/man/its_asymmetric.1" 2>/dev/null; then
    green "man documents its_asymmetric $sub"
  else
    red "man missing its_asymmetric subcommand $sub"
  fi
done

if [[ "$FAIL" -eq 0 ]]; then
  echo "verify_cli_completions.sh: ALL CHECKS PASSED"
else
  echo "verify_cli_completions.sh: SOME CHECKS FAILED" >&2
  exit 1
fi
