#!/usr/bin/env bash
# Install shell completions for constitution + optional ridge CLIs (bash/zsh/fish/ps1).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ECO_ROOT="$(cd "$ROOT/.." && pwd)"
KM="${ITS_KM_DIR:-$ECO_ROOT/ITS-KeyManagement}"
ASY="${ITS_ASYMMETRIC_DIR:-$ECO_ROOT/ITS-asymmetric}"

usage() {
  echo "Usage: $0 [--bash] [--zsh] [--fish] [--ps1] [--all] [--constitution-only]" >&2
  echo "  Default: detect shell and install matching completion." >&2
  echo "  Constitution: its-routing, its-km, its_asymmetric" >&2
  echo "  Optional ridges (when repos present): its_otm, its_timelock, sss_chain, its_fe, its_ledger" >&2
  exit 1
}

install_pair() {
  local name="$1" src_dir="$2"
  [[ -d "$src_dir" ]] || return 0
  cp "$src_dir/${name}.bash" "$BASH_DEST/${name}" 2>/dev/null || true
  cp "$src_dir/${name}.zsh" "$ZSH_DEST/_${name//-/_}" 2>/dev/null || true
  cp "$src_dir/${name}.fish" "$FISH_DEST/${name}.fish" 2>/dev/null || true
  cp "$src_dir/${name}.ps1" "$PS_DEST/${name}.ps1" 2>/dev/null || true
  echo "  installed: $name from $src_dir"
}

install_bash() {
  BASH_DEST="${BASH_COMPLETION_DIR:-/etc/bash_completion.d}"
  if [[ ! -w "$BASH_DEST" ]]; then
    BASH_DEST="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$BASH_DEST"
  fi
  ZSH_DEST="${ZSH_COMPLETION_DIR:-$HOME/.zsh/completions}"
  FISH_DEST="${FISH_COMPLETION_DIR:-$HOME/.config/fish/completions}"
  PS_DEST="${PS_COMPLETION_DIR:-$HOME/.its/completions}"
  mkdir -p "$ZSH_DEST" "$FISH_DEST" "$PS_DEST"

  install_pair its-routing "$ROOT/completions"
  install_pair its-km "$KM/completions"
  install_pair its_asymmetric "$ASY/completions"

  if [[ "$RIDGE" -eq 1 ]]; then
    for spec in \
      "its_otm:$ECO_ROOT/ITS-OTM_public_attestation/completions" \
      "its_timelock:$ECO_ROOT/ITS-self_enclosed_timelock/completions" \
      "sss_chain:$ECO_ROOT/SSS_CHAIN/completions" \
      "its_fe:$ECO_ROOT/ITS-fingerprint_erasure/completions" \
      "its_ledger:$ECO_ROOT/ITS-ledger/completions"; do
      name="${spec%%:*}"
      dir="${spec#*:}"
      install_pair "$name" "$dir"
    done
  fi
  echo "bash: $BASH_DEST"
}

install_zsh() {
  BASH_DEST=/dev/null
  ZSH_DEST="${ZSH_COMPLETION_DIR:-$HOME/.zsh/completions}"
  FISH_DEST="${FISH_COMPLETION_DIR:-$HOME/.config/fish/completions}"
  PS_DEST="${PS_COMPLETION_DIR:-$HOME/.its/completions}"
  mkdir -p "$ZSH_DEST" "$FISH_DEST" "$PS_DEST"
  install_pair its-routing "$ROOT/completions"
  install_pair its-km "$KM/completions"
  install_pair its_asymmetric "$ASY/completions"
  echo "zsh: $ZSH_DEST (add to fpath)"
}

install_fish() {
  BASH_DEST=/dev/null
  ZSH_DEST=/dev/null
  FISH_DEST="${FISH_COMPLETION_DIR:-$HOME/.config/fish/completions}"
  PS_DEST="${PS_COMPLETION_DIR:-$HOME/.its/completions}"
  mkdir -p "$FISH_DEST" "$PS_DEST"
  install_pair its-routing "$ROOT/completions"
  install_pair its-km "$KM/completions"
  install_pair its_asymmetric "$ASY/completions"
  echo "fish: $FISH_DEST"
}

install_ps1() {
  BASH_DEST=/dev/null
  ZSH_DEST=/dev/null
  FISH_DEST=/dev/null
  PS_DEST="${PS_COMPLETION_DIR:-$HOME/.its/completions}"
  mkdir -p "$PS_DEST"
  install_pair its-routing "$ROOT/completions"
  install_pair its-km "$KM/completions"
  install_pair its_asymmetric "$ASY/completions"
  echo "powershell: dot-source files in $PS_DEST"
}

RIDGE=1
do_bash=0 do_zsh=0 do_fish=0 do_ps1=0
if [[ $# -eq 0 ]]; then
  case "${SHELL:-}" in
    */zsh) do_zsh=1 ;;
    */fish) do_fish=1 ;;
    *) do_bash=1 ;;
  esac
else
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --bash) do_bash=1 ;;
      --zsh) do_zsh=1 ;;
      --fish) do_fish=1 ;;
      --ps1) do_ps1=1 ;;
      --all) do_bash=1; do_zsh=1; do_fish=1; do_ps1=1 ;;
      --constitution-only) RIDGE=0 ;;
      -h|--help) usage ;;
      *) usage ;;
    esac
    shift
  done
fi

[[ "$do_bash" -eq 1 ]] && install_bash
[[ "$do_zsh" -eq 1 ]] && install_zsh
[[ "$do_fish" -eq 1 ]] && install_fish
[[ "$do_ps1" -eq 1 ]] && install_ps1

echo "install_completions.sh: done (ITS_KM_DIR=$KM ITS_ASYMMETRIC_DIR=$ASY)"
