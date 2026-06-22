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

## Sprint 6 — `ecosystem-v1.0.0-complete` gate (M1–M20)

**Math gate** — `./scripts/verify_math.sh` (local run 2026-06-22: **green**):

- [x] **M1** — `lake build routing-math-cert` (0 `sorry` in cert closure)
- [x] **M2** — C1 wire Shannon via `Transport/WireComposition.lean` (no `mutualInfo := 0` stub)
- [x] **M3** — I(author; O) = 0 package (`AuthorAttributionZero.lean` in cert)
- [x] **M4** — O⁺ under P1–P3 + observation alphabet (`OplusClosure.lean`)
- [x] **M5** — EP split encryptor vs verify (`EndpointSplit.lean`)
- [x] **M6** — Offline / sneakernet channel (`OfflineChannel.lean`)
- [x] **M7** — Master unattackable certificate (`UnattackableCertificate.lean` smoke)
- [x] **M8** — Math-only verify script (`./scripts/verify_math.sh`)
- [x] **M9** — Finite MI (`Transport/FiniteMutualInfo.lean` smoke; no stub)
- [x] **M10** — `networkEcosystemCertificateV5` (`MasterTheorem.lean` smoke)
- [x] **M11** — 0 `sorry` in ROUTING + ITS-asymmetric mathematics
- [x] **M12** — OTM C2 integrity import (`Otm/OtmIntegrity.lean` smoke)
- [x] **M13** — `PROOF_MANIFEST.md` v9 CORE one-liner + finite-MI column
- [x] **M14** — C4 timelock deniability (`Stl/Security/Deniability.lean` smoke)
- [x] **M15** — Coercion model (`CoercionModel.lean` smoke)
- [x] **M16** — Cert path isolation (no dev-onion in `routing-math-cert`)
- [x] **M17** — `networkEcosystemCertificateV6–v9` (`MasterTheoremV6.lean` smoke) + `lake build routing-math-refinement` + Rust `rust_epoch_cell_refines_ideal`
- [x] **M18** — 0 `Prop := True` stubs + public mirror deploy (`pipe_its_http_pool_e2e.sh`, D9)
- [x] **M19** — ITS-A forward proof (`ForwardProof.lean` smoke) + KM pool + SOCKS egress (`pipe_its_km_pool_e2e.sh`, `pipe_its_socks_pool_e2e.sh`)
- [x] **M20** — ValidFwd / witness / receive gate (`ValidForwardParty.lean` smoke) + `cargo test -p its_routing valid_forward witness_consensus` + timelock pipe (`pipe_timelock.sh`) + censorship recovery (`pipe_its_censorship_recovery_e2e.sh`)

**Ecosystem gate** — `./scripts/verify_ecosystem.sh /home/user` (local run 2026-06-22 @ ROUTING `de1a7c5`: **ALL CHECKS PASSED**):

- [x] Dependency pins: 0x1F980 git tags only (no `path = "../"`)
- [x] ROUTING workspace + ITS-A unit tests green
- [x] M21 censorship / sneakernet / AEH pipes
- [x] M22 manifest alignment (`PROOF_MANIFEST.md` + `REFINEMENT_MANIFEST.md`)
- [x] Full `verify_ecosystem.sh` green on local monorepo (`/home/user`, ~17 min)
- [ ] Full `verify_ecosystem.sh` green on **tagged checkout** via Z10 fresh clone (see below)

**Z10 fresh-clone status** (2026-06-22):

- `./scripts/bootstrap.sh /tmp/its-ecosystem-z10-test` — initial attempt failed: remote `SSS_CHAIN` has no `master` branch; `bootstrap.sh` updated with `main` fallback.
- Re-run after merge: `ECOSYSTEM_TAG=v2.0.0 ./scripts/bootstrap.sh ./its-ecosystem && ./its-ecosystem/ITS-ROUTING/scripts/verify_math.sh && ./its-ecosystem/ITS-ROUTING/scripts/verify_ecosystem.sh ./its-ecosystem`
- Local monorepo gate is green; Z10 remains **operator action** until bootstrap + verify on isolated tree succeeds.

**Operator-only** (requires user confirmation — not automated):

- [ ] Clone all sibling repos at matching release tags — see [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)
- [ ] `./scripts/bootstrap.sh` → `verify_math.sh` + `verify_ecosystem.sh` on fresh clone (Z10)
- [ ] Independent reviewer sign-off on [PROOF_MANIFEST.md](PROOF_MANIFEST.md) v9 + [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md)
- [ ] Push all repos; apply `v1.0.0` per repo + meta-tag `ecosystem-v1.0.0-complete`

---

Badge: **ITS v0.10 verify_fast + Lean green — reproducible build**

Manual operator gates: standard-profile `adversary_* --ignored`, 1 MiB pipe (multi-hour on standard profile).

**Target badge (Sprint 6):** `ecosystem-v1.0.0-complete` — verify_math + verify_ecosystem + P8.* + tagged sibling repos.

---

## Sprint 7 — v10 implementation certificate (M23–M26)

**Math gate** — `./scripts/verify_math.sh` M23–M26:

- [ ] **M23** — `lake build routing-math-refinement` (all v10 roots: ValidFwd, witness, receive gate, client pool, SSS stub)
- [ ] **M24** — smoke `Refinement/ValidForwardRefinement.lean`
- [ ] **M25** — smoke `Refinement/WitnessConsensusRefinement.lean` + `Refinement/ForwardReceiveGateRefinement.lean`
- [ ] **M26** — smoke `networkImplementationCertificateV10` in `MasterTheoremV6.lean`; PROOF_MANIFEST v10 grep

**Refinement review:**

- [ ] 0 `sorry` in `mathematics/Refinement/*.lean`
- [ ] 0 `Prop := True` stubs in refinement modules
- [ ] `validForwardRefinementClosed`, `witnessConsensusRefinementClosed`, `forwardReceiveGateRefinementClosed`, `clientPoolRefinementClosed` proved
- [ ] `cargo test -p its_routing --lib valid_forward consensus` green
- [ ] REFINEMENT_MANIFEST.md truth table: Proved / smoke / Outside — no grey zone
- [ ] M21–M22 E2E pipes labeled smoke only in manifests

**v10 sign-off:**

- [ ] Independent reviewer confirms `networkImplementationCertificateV10` bundle
- [ ] RNG byte draw documented as Outside (option B) in REFINEMENT_MANIFEST
- [ ] v10.1 sibling tracks (asymmetric, OTM, timelock, SSS) tracked as planned
