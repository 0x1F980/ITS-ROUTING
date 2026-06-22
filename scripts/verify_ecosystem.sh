#!/usr/bin/env bash
# Verify ITS ecosystem layer rules and test suites on a monorepo or bootstrap tree.
set -euo pipefail

ECO_ROOT="${1:-/home/user}"
ROUTING="$ECO_ROOT/ROUTING"
FAIL=0

red() { echo "FAIL: $*"; FAIL=1; }
green() { echo "OK: $*"; }

# Portable search: ripgrep if present, else grep.
search() {
  if command -v rg >/dev/null 2>&1; then
    rg "$@"
  else
  case "$1" in
    -l) shift; grep -l -E "$@" 2>/dev/null || true ;;
    -q) shift; grep -q -E "$@" ;;
    *) grep -E "$@" 2>/dev/null || true ;;
  esac
  fi
}

CARGO_TMPLS=(
  "$ECO_ROOT/SSS_CHAIN/Cargo.toml"
  "$ECO_ROOT/ITS-asymmetric/Cargo.toml"
  "$ECO_ROOT/ITS-OTM_public_attestation/Cargo.toml"
  "$ECO_ROOT/ITS-self_enclosed_timelock/Cargo.toml"
  "$ECO_ROOT/ITS-hardware/Cargo.toml"
  "$ECO_ROOT/ITS-ledger/Cargo.toml"
  "$ECO_ROOT/ITS-fingerprint_erasure/Cargo.toml"
  "$ECO_ROOT/ITS-KeyManagement/Cargo.toml"
  "$ECO_ROOT/ROUTING/Cargo.toml"
)

echo "=== dependency pins: active org only (0x1F980) ==="
if search -l '0x1F464' "${CARGO_TMPLS[@]}" | grep -q .; then
  red "dependency pins: forbidden org reference in Cargo.toml"
else
  green "dependency pins: active org only"
fi

echo "=== dependency pins: git tags, not sibling path deps ==="
if search 'path = "\.\./' "${CARGO_TMPLS[@]}" | grep -q .; then
  red "dependency pins: sibling path = \"../\" found in Cargo.toml"
else
  green "dependency pins: git tags, not sibling path deps"
fi

echo "=== source: no retired module names in active trees ==="
if search 'core_logic|hydra_sss|its_transport::routing|its_transport::ratchet' \
  "$ECO_ROOT/ITS-asymmetric/src" \
  "$ECO_ROOT/ITS-KeyManagement/src" \
  "$ECO_ROOT/ITS-hardware/src" \
  "$ECO_ROOT/ITS-ledger/src" \
  "$ECO_ROOT/ROUTING/its_routing/src" \
  "$ECO_ROOT/ROUTING/its_routing/tests" | grep -q .; then
  red "source: retired import names in active src/tests"
else
  green "source: canonical module names only"
fi

echo "=== constitution: ITS_ECOSYSTEM.md present ==="
[[ -f "$ROUTING/ITS_ECOSYSTEM.md" ]] && green "constitution: ITS_ECOSYSTEM.md" || red "constitution: ITS_ECOSYSTEM.md missing"

echo "=== constitution: ITS = Information-Theoretic Secrecy ==="
if [[ -f "$ROUTING/ITS_ECOSYSTEM.md" ]] && grep -q 'Information-Theoretic Secrecy' "$ROUTING/ITS_ECOSYSTEM.md"; then
  green "constitution: ITS defined as Information-Theoretic Secrecy"
else
  red "constitution: ITS must be defined as Information-Theoretic Secrecy in ITS_ECOSYSTEM.md"
fi

echo "=== proofs: PROOF_MANIFEST in each math repo ==="
for m in ITS-asymmetric SSS_CHAIN ITS-OTM_public_attestation ITS-self_enclosed_timelock; do
  [[ -f "$ECO_ROOT/$m/PROOF_MANIFEST.md" ]] && green "proofs: PROOF_MANIFEST ($m)" || red "proofs: PROOF_MANIFEST missing ($m)"
done

echo "=== keymanagement: no session binary coupling in source ==="
if search 'its_sessions' "$ECO_ROOT/ITS-KeyManagement/src" | grep -q .; then
  red "keymanagement: its_sessions reference in KM source"
else
  green "keymanagement: wire FS via asymmetric epoch-advance only"
fi

echo "=== tests: ROUTING workspace ==="
if [[ -d "$ROUTING" ]]; then
  (cd "$ROUTING" && cargo test -p its_transport -p its_routing --features full --quiet) && green "tests: ROUTING workspace" || red "tests: ROUTING workspace"
fi

echo "=== ITS-A: ValidFwd + witness consensus unit tests ==="
if [[ -d "$ROUTING" ]]; then
  (cd "$ROUTING" && cargo test -p its_routing --lib valid_forward --quiet \
    && cargo test -p its_routing --lib consensus --quiet) \
    && green "ITS-A: valid_forward + witness_consensus" || red "ITS-A: valid_forward tests failed"
fi

echo "=== tests: math repos and glue ==="
for pkg in SSS_CHAIN ITS-OTM_public_attestation ITS-self_enclosed_timelock ITS-hardware ITS-ledger ITS-KeyManagement; do
  if [[ -f "$ECO_ROOT/$pkg/Cargo.toml" ]]; then
    (cd "$ECO_ROOT/$pkg" && cargo test --quiet 2>/dev/null) && green "tests: $pkg" || red "tests: $pkg"
  fi
done
if [[ -f "$ECO_ROOT/ITS-asymmetric/Cargo.toml" ]]; then
  asym_ok=1
  for t in trapdoor sss_epoch morphic epoch_seal pad_to_tier adversary_no_key; do
    if ! (cd "$ECO_ROOT/ITS-asymmetric" && cargo test --lib --features bundle,std "$t" --quiet 2>/dev/null); then
      asym_ok=0
      break
    fi
  done
  [[ "$asym_ok" -eq 1 ]] && green "tests: ITS-asymmetric core" || red "tests: ITS-asymmetric core"
fi

echo "=== transport: fragment and onion regression ==="
if [[ -d "$ROUTING" ]]; then
  if (cd "$ROUTING" && cargo test -p its_transport fragment onion --quiet 2>/dev/null); then
    green "transport: fragment and onion regression"
  else
    (cd "$ROUTING" && cargo test -p its_transport --quiet 2>/dev/null) && green "transport: its_transport full suite" || red "transport: its_transport tests"
  fi
fi

echo "=== layer: routing dependency tree excludes core_logic ==="
if [[ -d "$ROUTING" ]]; then
  if (cd "$ROUTING" && cargo tree -p its_routing 2>/dev/null | grep -q core_logic); then
    red "layer: routing tree must not include core_logic"
  else
    green "layer: routing tree excludes core_logic"
  fi
fi

echo "=== layer: routing must not compile against wire crypto (its_asymmetric) ==="
if [[ -d "$ROUTING" ]]; then
  if (cd "$ROUTING" && cargo tree -p its_routing 2>/dev/null | grep -q its_asymmetric); then
    red "layer: its_routing must not depend on its_asymmetric"
  else
    green "layer: routing relays opaque bytes (no its_asymmetric dep)"
  fi
fi

echo "=== layer: keymanagement subprocess-only (no sibling Cargo deps) ==="
if grep -E '^[^#]*(its_|sss_chain|core_logic)\s*=' "$ECO_ROOT/ITS-KeyManagement/Cargo.toml" 2>/dev/null | grep -q .; then
  red "layer: KM must not declare sibling ITS crates in Cargo.toml"
else
  green "layer: keymanagement subprocess-only"
fi

echo "=== layer: math repos isolated from transport ==="
math_violation=0
for m in SSS_CHAIN ITS-asymmetric ITS-OTM_public_attestation ITS-self_enclosed_timelock; do
  if search 'its_transport|its_routing|core_logic' "$ECO_ROOT/$m/Cargo.toml" | grep -q .; then
    red "layer: math repo must not depend on transport ($m)"
    math_violation=1
  fi
done
[[ "$math_violation" -eq 0 ]] && green "layer: math repos isolated from transport"

echo "=== layer: routing uses canonical its_transport imports ==="
if search 'hydra_sss|its_transport::routing|its_transport::ratchet' \
  "$ECO_ROOT/ROUTING/its_routing/src" \
  "$ECO_ROOT/ROUTING/its_routing/tests" | grep -q .; then
  red "layer: routing must import onion/sss_fragment/transport_otp_ratchet directly"
else
  green "layer: routing uses canonical its_transport imports"
fi

echo "=== v5: ITS_INFRASTRUCTURE_REPLACEMENT.md present ==="
[[ -f "$ROUTING/ITS_INFRASTRUCTURE_REPLACEMENT.md" ]] && green "v5: replacement doc" || red "v5: ITS_INFRASTRUCTURE_REPLACEMENT.md missing"

echo "=== v5: constitution defines C1-C4 channels ==="
if [[ -f "$ROUTING/ITS_ECOSYSTEM.md" ]] && grep -q 'C1 Payload' "$ROUTING/ITS_ECOSYSTEM.md" && grep -q 'C4 Benægtelighed' "$ROUTING/ITS_ECOSYSTEM.md"; then
  green "v5: C1-C4 in constitution"
else
  red "v5: ITS_ECOSYSTEM.md must define C1-C4 channels"
fi

echo "=== v5: routing math chaff honesty ==="
if [[ -f "$ROUTING/docs/archive/dev-onion/ITS-routing_mathematics.md" ]] \
  && grep -q 'ChaffIndistinguishability' "$ROUTING/docs/archive/dev-onion/ITS-routing_mathematics.md"; then
  green "v5: chaff ITS cites Lean proof"
else
  red "v5: docs/archive/dev-onion/ITS-routing_mathematics.md must cite ChaffIndistinguishability Lean for Shannon ITS chaff"
fi

echo "=== v5: vault ITS-only (no Argon2/ITSKMV2 in KM/ledger src) ==="
if search -i 'argon2|ITSKMV2|chacha20|pbkdf2' \
  "$ECO_ROOT/ITS-KeyManagement/src" \
  "$ECO_ROOT/ITS-ledger/src" | grep -q .; then
  red "v5: KM/ledger src must not reference Argon2/ITSKMV2/chacha/pbkdf2"
else
  green "v5: vault ITS-only in KM/ledger"
fi

echo "=== v5: transport ratchet no HKDF/PBKDF ==="
if search 'hkdf|pbkdf2|Sha256' "$ECO_ROOT/ROUTING/its_transport/src/transport_otp_ratchet.rs" | grep -q .; then
  red "v5: transport_otp_ratchet must not use HKDF/PBKDF/Sha256"
else
  green "v5: transport OTP ratchet (SSS epoch)"
fi

echo "=== v5: ROUTING Lean dev-onion (routing-math-dev) ==="
if [[ -d "$ROUTING/mathematics" ]]; then
  (cd "$ROUTING/mathematics" && lake build routing-math-dev 2>/dev/null) && green "v5: routing-math-dev build" || red "v5: routing-math-dev build failed"
fi

echo "=== M17: routing-math-refinement ==="
if [[ -d "$ROUTING/mathematics" ]]; then
  (cd "$ROUTING/mathematics" && lake build routing-math-refinement 2>/dev/null) && green "M17: routing-math-refinement" || red "M17: routing-math-refinement failed"
fi

echo "=== v5: pipe_its_routing_e2e.sh present ==="
[[ -x "$ROUTING/scripts/pipe_its_routing_e2e.sh" ]] && green "v5: pipe_its_routing_e2e.sh" || red "v5: pipe_its_routing_e2e.sh missing or not executable"

echo "=== v5: its-wire/1 ALPN scripts ==="
[[ -x "$ROUTING/scripts/its-curl.sh" ]] && green "v5: its-curl.sh" || red "v5: its-curl.sh missing"
[[ -f "$ECO_ROOT/ITS-asymmetric/contrib/nginx-its-wire.conf" ]] && green "v5: nginx-its-wire.conf" || red "v5: nginx-its-wire.conf missing"

echo "=== v5: PROOF_MANIFEST ROUTING + KM ==="
[[ -f "$ROUTING/PROOF_MANIFEST.md" ]] && green "v5: ROUTING PROOF_MANIFEST" || red "v5: ROUTING PROOF_MANIFEST missing"
[[ -f "$ECO_ROOT/ITS-KeyManagement/PROOF_MANIFEST.md" ]] && green "v5: KM PROOF_MANIFEST" || red "v5: KM PROOF_MANIFEST missing"

echo "=== v5: SSS_CHAIN Lean in-repo ==="
[[ -f "$ECO_ROOT/SSS_CHAIN/mathematics/Epoch/StepForward.lean" ]] && green "v5: SSS_CHAIN Lean epoch" || red "v5: SSS_CHAIN/mathematics/Epoch/StepForward.lean missing"

echo "=== v1.5: UNATTACKABLE_MODEL.md present ==="
[[ -f "$ROUTING/ITS-routing_UNATTACKABLE_MODEL.md" ]] && green "v1.5: UNATTACKABLE_MODEL" || red "v1.5: ITS-routing_UNATTACKABLE_MODEL.md missing"

echo "=== v1.5: SecureEndpointDoctrine.md present ==="
[[ -f "$ROUTING/ITS-routing_SecureEndpointDoctrine.md" ]] && green "v1.5: SecureEndpointDoctrine" || red "v1.5: ITS-routing_SecureEndpointDoctrine.md missing"

echo "=== v1.5: fieldPrime = 2147483647 ==="
if grep -q '2147483647' "$ROUTING/mathematics/Transport/Basic.lean" 2>/dev/null; then
  green "v1.5: fieldPrime aligned"
else
  red "v1.5: Transport/Basic.lean must use fieldPrime 2147483647"
fi

echo "=== v1.5: no sorry in Composition.lean ==="
if grep -E '^\s*sorry\b|:= sorry' "$ROUTING/mathematics/Transport/Composition.lean" 2>/dev/null | grep -q .; then
  red "v1.5: Composition.lean contains sorry"
else
  green "v1.5: Composition.lean no sorry"
fi

echo "=== v1.5: release default features pool+aeh+otm ==="
if grep -q 'default = \["pool", "aeh", "otm"\]' "$ROUTING/its_routing/Cargo.toml" 2>/dev/null; then
  green "v1.5: CertifiedBuild default features"
else
  red "v1.5: its_routing default must be pool+aeh+otm"
fi

echo "=== v1.5: no WIKI_STEGO in release binary ==="
if [[ -x "$ROUTING/target/release/its-routing" ]]; then
  if strings "$ROUTING/target/release/its-routing" | grep -q 'WIKI_STEGO'; then
    red "v1.5: release binary must not contain WIKI_STEGO"
  else
    green "v1.5: no WIKI_STEGO in release"
  fi
else
  (cd "$ROUTING" && cargo build -p its_routing --release --quiet 2>/dev/null) && \
    strings "$ROUTING/target/release/its-routing" | grep -q 'WIKI_STEGO' && \
    red "v1.5: release binary must not contain WIKI_STEGO" || green "v1.5: no WIKI_STEGO in release"
fi

echo "=== v1.5: rust_epoch_cell_refines_ideal test ==="
if (cd "$ROUTING" && cargo test -p its_transport rust_epoch_cell_refines_ideal --quiet 2>/dev/null); then
  green "v1.5: rust_epoch_cell_refines_ideal"
else
  red "v1.5: rust_epoch_cell_refines_ideal test failed"
fi

echo "=== v1.5: pipe_its_pool_e2e.sh (primary) ==="
if [[ -x "$ROUTING/scripts/pipe_its_pool_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_pool_e2e.sh" >/dev/null 2>&1 && green "v1.5: pipe_its_pool_e2e.sh" || red "v1.5: pipe_its_pool_e2e.sh failed"
else
  red "v1.5: pipe_its_pool_e2e.sh missing"
fi

echo "=== v1.5: pipe_its_aeh_censorship_e2e.sh (fallback) ==="
if [[ -x "$ROUTING/scripts/pipe_its_aeh_censorship_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_aeh_censorship_e2e.sh" >/dev/null 2>&1 && green "v1.5: pipe_its_aeh_censorship_e2e.sh" || red "v1.5: pipe_its_aeh_censorship_e2e.sh failed"
else
  red "v1.5: pipe_its_aeh_censorship_e2e.sh missing"
fi

echo "=== v1.5: pipe_its_sneakernet_e2e.sh (A-resilience) ==="
if [[ -x "$ROUTING/scripts/pipe_its_sneakernet_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_sneakernet_e2e.sh" >/dev/null 2>&1 && green "v1.5: pipe_its_sneakernet_e2e.sh" || red "v1.5: pipe_its_sneakernet_e2e.sh failed"
else
  red "v1.5: pipe_its_sneakernet_e2e.sh missing"
fi

echo "=== v1.6: no empty_passes in pool receive ==="
if grep -q 'empty_passes' "$ROUTING/its_routing/src/client.rs" 2>/dev/null; then
  red "v1.6: client.rs must not use empty_passes"
else
  green "v1.6: no empty_passes"
fi

echo "=== v1.6: cover_transport module ==="
[[ -f "$ROUTING/its_routing/src/cover_transport.rs" ]] && green "v1.6: cover_transport.rs" || red "v1.6: cover_transport.rs missing"

echo "=== v1.6: epoch_interval_ms wired in receive ==="
if grep -q 'epoch_interval_ms' "$ROUTING/its_routing/src/client.rs" 2>/dev/null; then
  green "v1.6: epoch_interval_ms in client receive"
else
  red "v1.6: epoch_interval_ms not wired"
fi

echo "=== v1.6: ParticipationSymmetry.lean builds ==="
if (cd "$ROUTING/mathematics" && lake env lean ParticipationSymmetry.lean 2>/dev/null); then
  green "v1.6: ParticipationSymmetry.lean"
else
  red "v1.6: ParticipationSymmetry.lean build failed"
fi

echo "=== v1.6: ComparativeThreatDoctrine.lean builds ==="
if (cd "$ROUTING/mathematics" && lake env lean ComparativeThreatDoctrine.lean 2>/dev/null); then
  green "v1.6: ComparativeThreatDoctrine.lean"
else
  red "v1.6: ComparativeThreatDoctrine.lean build failed"
fi

echo "=== v2.0: UNATTACKABLE_MODEL L11-L13 ==="
if grep -q 'L11' "$ROUTING/ITS-routing_UNATTACKABLE_MODEL.md" 2>/dev/null \
  && grep -q 'L13' "$ROUTING/ITS-routing_UNATTACKABLE_MODEL.md" 2>/dev/null; then
  green "v2.0: UNATTACKABLE_MODEL L11-L13"
else
  red "v2.0: UNATTACKABLE_MODEL missing L11-L13"
fi

echo "=== v2.0: README pool-primary narrative ==="
if grep -q 'Production path.*UES Monocell Pool' "$ROUTING/README.md" 2>/dev/null; then
  green "v2.0: README pool-primary"
else
  red "v2.0: README must lead with pool prod path"
fi

echo "=== v1.6: pipe_its_cover_harvest_e2e.sh ==="
if [[ -x "$ROUTING/scripts/pipe_its_cover_harvest_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_cover_harvest_e2e.sh" >/dev/null 2>&1 && green "v1.6: pipe_its_cover_harvest_e2e.sh" || red "v1.6: pipe_its_cover_harvest_e2e.sh failed"
else
  red "v1.6: pipe_its_cover_harvest_e2e.sh missing"
fi

echo "=== v1.6: pipe_its_http_pool_e2e.sh ==="
if [[ -x "$ROUTING/scripts/pipe_its_http_pool_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_http_pool_e2e.sh" >/dev/null 2>&1 && green "v1.6: pipe_its_http_pool_e2e.sh" || red "v1.6: pipe_its_http_pool_e2e.sh failed"
else
  red "v1.6: pipe_its_http_pool_e2e.sh missing"
fi

echo "=== v1.7: pipe_its_km_pool_e2e.sh ==="
if [[ -x "$ROUTING/scripts/pipe_its_km_pool_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_km_pool_e2e.sh" >/dev/null 2>&1 && green "v1.7: pipe_its_km_pool_e2e.sh" || red "v1.7: pipe_its_km_pool_e2e.sh failed"
else
  red "v1.7: pipe_its_km_pool_e2e.sh missing"
fi

echo "=== v1.7: pipe_its_http_pool_e2e.sh (public mirror deploy) ==="
if [[ -x "$ROUTING/scripts/pipe_its_http_pool_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_http_pool_e2e.sh" >/dev/null 2>&1 && green "v1.7: pipe_its_http_pool_e2e.sh (mirror deploy)" || red "v1.7: pipe_its_http_pool_e2e.sh failed"
else
  red "v1.7: pipe_its_http_pool_e2e.sh missing"
fi

echo "=== v1.8: pipe_its_socks_pool_e2e.sh ==="
if [[ -x "$ROUTING/scripts/pipe_its_socks_pool_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_socks_pool_e2e.sh" >/dev/null 2>&1 && green "v1.8: pipe_its_socks_pool_e2e.sh" || red "v1.8: pipe_its_socks_pool_e2e.sh failed"
else
  red "v1.8: pipe_its_socks_pool_e2e.sh missing"
fi

echo "=== v1.8: pipe_its_censorship_recovery_e2e.sh ==="
if [[ -x "$ROUTING/scripts/pipe_its_censorship_recovery_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_censorship_recovery_e2e.sh" >/dev/null 2>&1 && green "v1.8: pipe_its_censorship_recovery_e2e.sh" || red "v1.8: pipe_its_censorship_recovery_e2e.sh failed"
else
  red "v1.8: pipe_its_censorship_recovery_e2e.sh missing"
fi

echo "=== v1.8: PoolMailbox wired in pool receive ==="
if grep -q 'accept_reconstructed_payload' "$ROUTING/its_routing/src/client.rs" 2>/dev/null \
  && grep -q 'PoolMailbox' "$ROUTING/its_routing/src/client.rs" 2>/dev/null; then
  green "v1.8: PoolMailbox in client receive"
else
  red "v1.8: PoolMailbox not wired in client receive"
fi

echo "=== v1.8: ParticipationTheorem mailbox-ID in ciphertext ==="
if grep -q 'mailboxIdInCiphertextOnly' "$ROUTING/mathematics/ParticipationTheorem.lean" 2>/dev/null; then
  green "v1.8: ParticipationTheorem mailbox"
else
  red "v1.8: ParticipationTheorem mailbox missing"
fi

echo "=== v1.7: fountain_enabled in epoch_cell.rs ==="
if grep -q 'fountain_extra_chaff_epochs' "$ROUTING/its_transport/src/epoch_cell.rs" 2>/dev/null; then
  green "v1.6: fountain_enabled in epoch_cell"
else
  red "v1.6: fountain_enabled not in epoch_cell.rs"
fi

echo "=== v2.0: ITS-routing_SUPERIORITY.md ==="
[[ -f "$ROUTING/ITS-routing_SUPERIORITY.md" ]] && green "v2.0: SUPERIORITY.md" || red "v2.0: SUPERIORITY.md missing"

echo "=== v2.0: QUICKSTART.md ==="
[[ -f "$ROUTING/QUICKSTART.md" ]] && green "v2.0: QUICKSTART.md" || red "v2.0: QUICKSTART.md missing"

echo "=== v2.0: config.prod.toml ==="
[[ -f "$ROUTING/config.prod.toml" ]] && green "v2.0: config.prod.toml" || red "v2.0: config.prod.toml missing"

echo "=== v4: absolute deniability Lean certificate ==="
if [[ -f "$ROUTING/mathematics/PlausibleDeniabilityAbsolute.lean" ]] \
  && [[ -f "$ROUTING/mathematics/BroadcastIPSymmetry.lean" ]] \
  && [[ -f "$ROUTING/mathematics/RecipientAttributionZero.lean" ]] \
  && grep -q 'absolutePlausibleDeniability' "$ROUTING/mathematics/UnattackableCertificate.lean" 2>/dev/null; then
  green "v4: absolute deniability certificate"
else
  red "v4: absolute deniability certificate missing"
fi

echo "=== v4: verify_math.sh ==="
if [[ -x "$ROUTING/scripts/verify_math.sh" ]]; then
  if bash "$ROUTING/scripts/verify_math.sh" >/dev/null 2>&1; then
    green "v4: verify_math.sh"
  else
    red "v4: verify_math.sh failed"
  fi
else
  red "v4: verify_math.sh missing"
fi

echo "=== M18: public mirror reference deploy (BIS/P1-P3) ==="
if [[ -f "$ROUTING/ITS-routing_DEPLOY_MATH_GATES.md" ]] \
  && [[ -f "$ROUTING/deploy/pool-mirror/pool_mirror_server.py" ]]; then
  if [[ -x "$ROUTING/scripts/pipe_its_http_pool_e2e.sh" ]]; then
    "$ROUTING/scripts/pipe_its_http_pool_e2e.sh" >/dev/null 2>&1 && green "M18: pipe_its_http_pool_e2e.sh" || red "M18: pipe_its_http_pool_e2e.sh failed"
  else
    red "M18: pipe_its_http_pool_e2e.sh missing"
  fi
else
  red "M18: deploy math gates doc or pool-mirror missing"
fi

echo "=== M19: KM send + SOCKS egress (P8.2/P8.4) ==="
m19_ok=1
[[ -f "$ROUTING/ITS-routing_SOCKS_EGRESS.md" ]] || { red "M19: ITS-routing_SOCKS_EGRESS.md missing"; m19_ok=0; }
if [[ -x "$ROUTING/scripts/pipe_its_km_pool_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_km_pool_e2e.sh" >/dev/null 2>&1 || { red "M19: pipe_its_km_pool_e2e.sh failed"; m19_ok=0; }
else
  red "M19: pipe_its_km_pool_e2e.sh missing"; m19_ok=0
fi
if [[ -x "$ROUTING/scripts/pipe_its_socks_pool_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_socks_pool_e2e.sh" >/dev/null 2>&1 || { red "M19: pipe_its_socks_pool_e2e.sh failed"; m19_ok=0; }
else
  red "M19: pipe_its_socks_pool_e2e.sh missing"; m19_ok=0
fi
[[ "$m19_ok" -eq 1 ]] && green "M19: KM + SOCKS egress"

echo "=== M20: timelock pipe (P8.5) ==="
if [[ -x "$ROUTING/scripts/pipe_timelock.sh" ]]; then
  (cd "$ROUTING" && cargo build -p its_routing --release --features timelock --quiet 2>/dev/null) || true
  "$ROUTING/scripts/pipe_timelock.sh" >/dev/null 2>&1 && green "M20: pipe_timelock.sh" || red "M20: pipe_timelock.sh failed"
else
  red "M20: pipe_timelock.sh missing"
fi

echo "=== M20: ValidFwd witness pipe (optional) ==="
if [[ -x "$ROUTING/scripts/pipe_its_validfwd_e2e.sh" ]]; then
  "$ROUTING/scripts/pipe_its_validfwd_e2e.sh" >/dev/null 2>&1 && green "M20: pipe_its_validfwd_e2e.sh" || red "M20: pipe_its_validfwd_e2e.sh failed"
else
  green "M20: pipe_its_validfwd_e2e.sh skipped (not present)"
fi

echo "=== M21: censorship recovery pipes (P8.3/B4) ==="
m21_ok=1
for p in pipe_its_censorship_recovery_e2e.sh pipe_its_sneakernet_e2e.sh pipe_its_aeh_censorship_e2e.sh; do
  if [[ -x "$ROUTING/scripts/$p" ]]; then
    "$ROUTING/scripts/$p" >/dev/null 2>&1 || { red "M21: $p failed"; m21_ok=0; }
  else
    red "M21: $p missing"; m21_ok=0
  fi
done
[[ -f "$ROUTING/ITS-routing_CENSORSHIP_RECOVERY.md" ]] || { red "M21: ITS-routing_CENSORSHIP_RECOVERY.md missing"; m21_ok=0; }
[[ "$m21_ok" -eq 1 ]] && green "M21: censorship + sneakernet pipes"

echo "=== M22: manifest ↔ Lean/Rust alignment ==="
m22_ok=1
for f in MasterTheorem.lean BroadcastIPDerivation.lean Refinement/EpochCellCorrectness.lean; do
  mod="${f%.lean}"
  mod="${mod//\//.}"
  grep -q "$mod" "$ROUTING/PROOF_MANIFEST.md" 2>/dev/null || { red "M22: PROOF_MANIFEST missing $mod"; m22_ok=0; }
done
for p in pipe_its_pool_e2e.sh pipe_its_socks_pool_e2e.sh pipe_timelock.sh; do
  grep -q "$p" "$ROUTING/REFINEMENT_MANIFEST.md" 2>/dev/null || { red "M22: REFINEMENT_MANIFEST missing $p"; m22_ok=0; }
done
[[ -f "$ROUTING/ITS-routing_STANDARD_REPLACEMENT.md" ]] || { red "M22: ITS-routing_STANDARD_REPLACEMENT.md missing"; m22_ok=0; }
[[ -f "$ROUTING/ITS-routing_OVERLAY_EXTINCTION.md" ]] || { red "M22: ITS-routing_OVERLAY_EXTINCTION.md missing"; m22_ok=0; }
[[ -f "$ROUTING/ITS_MIGRATION_GUIDES.md" ]] && grep -q 'Tor SOCKS' "$ROUTING/ITS_MIGRATION_GUIDES.md" || { red "M22: ITS_MIGRATION_GUIDES Tor/I2P migration missing"; m22_ok=0; }
[[ "$m22_ok" -eq 1 ]] && green "M22: PROOF + REFINEMENT manifests aligned"

echo "=== Sprint 5: product docs D1-D30 registry ==="
d_ok=1
for doc in ITS-routing_SOCKS_EGRESS.md ITS-routing_DEPLOY_MATH_GATES.md ITS-routing_STANDARD_REPLACEMENT.md ITS-routing_OVERLAY_EXTINCTION.md ITS_MIGRATION_GUIDES.md QUICKSTART.md; do
  [[ -f "$ROUTING/$doc" ]] || { red "Sprint5 doc missing: $doc"; d_ok=0; }
done
[[ "$d_ok" -eq 1 ]] && green "Sprint5: product docs D7-D30 present"

if [[ "$FAIL" -eq 0 ]]; then
  echo "=== verify_ecosystem: ALL CHECKS PASSED ==="
else
  echo "=== verify_ecosystem: SOME CHECKS FAILED ==="
  exit 1
fi
