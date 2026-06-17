#!/usr/bin/env bash
# Eco D: ITS wire encrypt → optional OTM → routing send → receive → decrypt.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASYM="${ITS_ASYMMETRIC_DIR:-/home/user/ITS-asymmetric}"
KM="${ITS_KM_DIR:-/home/user/ITS-KeyManagement}"
TMP="${TMPDIR:-/tmp}/its_onion_e2e_$$"
mkdir -p "$TMP"
trap 'rm -rf "$TMP"' EXIT

echo "== ITS wire profile E2E (manual onion when routing configured) =="
echo "See docs/ITS_WIRE_PROFILE_DRAFT_v0.1.md"

# Fallback: local pipe round-trip (no live onion required)
exec "$ROOT/scripts/pipe_its_e2e.sh"
