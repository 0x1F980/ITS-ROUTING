#!/usr/bin/env bash
# Install its-routing + its-km shell completions (bash/zsh/fish/ps1).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KM="${ITS_KM_DIR:-/home/user/ITS-KeyManagement}"

usage() {
  echo "Usage: $0 [--bash] [--zsh] [--fish] [--ps1] [--all]" >&2
  echo "  Default: detect shell and install matching completion." >&2
  exit 1
}

install_bash() {
  local dest="${BASH_COMPLETION_DIR:-/etc/bash_completion.d}"
  if [[ ! -w "$dest" ]]; then
    dest="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$dest"
  fi
  cp "$ROOT/completions/its-routing.bash" "$dest/its-routing"
  cp "$KM/completions/its-km.bash" "$dest/its-km"
  echo "bash: $dest/its-routing $dest/its-km"
}

install_zsh() {
  local dest="${ZSH_COMPLETION_DIR:-$HOME/.zsh/completions}"
  mkdir -p "$dest"
  cp "$ROOT/completions/its-routing.zsh" "$dest/_its-routing"
  cp "$KM/completions/its-km.zsh" "$dest/_its_km"
  echo "zsh: $dest/_its-routing $dest/_its_km (add dest to fpath)"
}

install_fish() {
  local dest="${FISH_COMPLETION_DIR:-$HOME/.config/fish/completions}"
  mkdir -p "$dest"
  cp "$ROOT/completions/its-routing.fish" "$dest/its-routing.fish"
  cp "$KM/completions/its-km.fish" "$dest/its-km.fish"
  echo "fish: $dest/its-routing.fish $dest/its-km.fish"
}

install_ps1() {
  local dest="${PS_COMPLETION_DIR:-$HOME/.its/completions}"
  mkdir -p "$dest"
  cp "$ROOT/completions/its-routing.ps1" "$dest/"
  cp "$KM/completions/its-km.ps1" "$dest/"
  echo "powershell: $dest/its-routing.ps1 $dest/its-km.ps1"
  echo "  dot-source: . $dest/its-routing.ps1"
}

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

echo "install_completions.sh: done"
