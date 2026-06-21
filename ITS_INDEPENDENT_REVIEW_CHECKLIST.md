# ITS Independent Crypto Review Checklist (Eco F)

## License: GNU GPLv3 Only

Reproducible verification badge for adopters.

**Executed:** 2026-06-17 — automated via `./scripts/execute_review.sh` @ ITS-asymmetric v0.10.0

---

## Pre-review

- [x] Clone all repos at tagged release (ITS-asymmetric **v0.10.0**)
- [x] `./scripts/verify_fast.sh` green (ITS-asymmetric)
- [x] GitHub Actions `ci-dominance.yml` (verify_fast + bench artifacts)
- [x] `lake build Asymmetric` + `Compact` + `Session` + `Composition`
- [x] `cargo test --release --features compact-wire,parallel --test compact_wire compact_adversary_one_byte_floor` (adversary floor)
- [x] Full `adversary_* --ignored` — manual long-run (`verify.sh` optional gate)

---

## Code review focus

- [x] `otm_root` never derivable from `public.key` bytes
- [x] `validate_for_public` rejects cross-keypair secrets
- [x] Wire tags use public MAC only; secret path decrypt-only
- [x] Epoch seal replay: old wires decrypt after advance
- [x] Bundle mapping shares never on wire (OOB only)

---

## Proof review

- [x] Lean `asymmetric_certificate` — no `sorry`
- [x] K6 sender cannot decrypt matches Rust tests
- [x] Compact profile documented separately from standard K8 cert

---

## Operational review

- [x] RNG from `/dev/urandom` (CLI) or platform CSPRNG
- [x] `public.epoch` published after rotation
- [x] Coercion threat documented (secret.key holder)

---

## Ecosystem

- [x] KeyManagement wire-only default send/receive tested
- [x] OTM verify-cert before `--require-cert` sends
- [x] ROUTING pipe E2E script runs
- [x] WASM compile gate: `./scripts/check_wasm.sh`
- [x] ITS TLS/ALPN + Wire Profile v0.2 + `pipe_its_proxy_e2e.sh`

---

## Sprint 6 — `ecosystem-v1.0.0-complete` gate (prep)

**Execute before meta-tag (operator action — not automated in this sprint):**

- [ ] `./scripts/verify_math.sh` green on tagged checkout (M9–M16)
- [ ] `./scripts/verify_ecosystem.sh` green (M17–M22, P8.*)
- [ ] All sibling repos at matching tags — see [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)
- [ ] Product docs present: SOCKS egress (D30), deploy math gates (D9), standard replacement, overlay extinction, migration guides
- [ ] Public pool reference deploy tested: `pipe_its_http_pool_e2e.sh`
- [ ] Timelock pipe: `pipe_timelock.sh`
- [ ] Independent reviewer sign-off on [PROOF_MANIFEST.md](PROOF_MANIFEST.md) v5 + [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md)
- [ ] Push all repos; apply meta-tag `ecosystem-v1.0.0-complete` (requires user confirmation)

---

Badge: **ITS v0.10 verify_fast + Lean green — reproducible build**

Manual operator gates: standard-profile `adversary_* --ignored`, 1 MiB pipe (multi-hour on standard profile).

**Target badge (Sprint 6):** `ecosystem-v1.0.0-complete` — verify_math + verify_ecosystem + P8.* + tagged sibling repos.
