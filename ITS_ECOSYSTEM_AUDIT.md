# ITS Ecosystem Audit: Doc-Sandhed, ITS, og Død Kode

**Dato:** 2026-06-14  
**Fix status:** **Fixed** — 2026-06-16 (ecosystem audit fix plan) · **Dominance pass** 2026-06-17 (v0.10.0 wire epoch, compact-wire Lean, KM `--wire-only`, ROUTING pipe E2E)  
**Scope:** 6 sibling repos  
**Primær læsekilde:** `*_SECURITY_LAYERS.md`, `*_FORMAL_VERIFICATION.md`, `*_mathematics.md` + Rust/Lean

| Repo | Path |
|------|------|
| ITS-asymmetric | `/home/user/ITS-asymmetric` |
| ITS-OTM | `/home/user/ITS-OTM_public_attestation` |
| ITS-timelock | `/home/user/ITS-self_enclosed_timelock` |
| SSS_CHAIN | `/home/user/SSS_CHAIN` |
| ITS-KeyManagement (KeyManagement) | `/home/user/ITS-KeyManagement` |
| ROUTING | `/home/user/ROUTING` |

---

## Changelog (2026-06-16 fix)

| Area | Change |
|------|--------|
| **ROUTING** | Removed unwired `anomaly_detection.rs`; docs no longer claim self-healing / integration simulation tests; added `tests/timelock_integration.rs`, `tests/otm_verify_integration.rs` |
| **ITS-asymmetric** | Wire v6-only purge (Rust + Lean); deleted `masking.rs`, `trapdoor_solve.rs`, v5 Lean modules; `V6_KEY_DRAW_MASK=0xFFF` in all adversary tests; docs v6-only |
| **ITS-OTM** | Removed dead `trapdoor` mod; added `test_wrong_tag_fails`, `test_tampered_forward_share_fails`; golden `T=4578` in M31 flow test |
| **SSS_CHAIN** | FORMAL_VERIFICATION test count 10; added `validate_fails_without_root`, `backward_transition_underdetermination` |
| **ITS-timelock** | W4.3 doc aligned; vision self-enclosed ITS; `lagrange_interpolate` from `sss_chain`; JSON fixture loaded in golden test; stronger deny test; `tests/cli_integration.rs` |
| **Lean Sacx** | `lake build Sacx` green (v6-only; `MorphicV6` inlined) |

All six repos: `cargo test --all-targets` green (assymetric adversary tests ~40 min in debug).

---

## Executive Summary

| Repo | Kode matcher ITS-dokumentation? | Kode matcher al dokumentation? | Død/legacy-kode |
|------|--------------------------------|--------------------------------|-----------------|
| **ITS-asymmetric** | **Ja** (version-free wire 0.9.1) | **Ja** | **Purged** (v1–v5 + stubs removed); 0.9.1 terminology cleanup + Lean aligned to 32-byte pk |
| **ITS-OTM** | **Ja** (design) | **Ja** | Minimal |
| **ITS-timelock** | **Ja** (self-enclosed ITS via SSS-chaining) | **Ja** | Minimal |
| **SSS_CHAIN** | **Ja** | **Ja** | Næsten ren |
| **ITS-KeyManagement** | **Ja** (vault/OTM path) | **Ja** (legacy bevidst dokumenteret) | Shims + fingerprint (fase 6) |
| **ROUTING** | **Delegerer korrekt** | **Ja** | Anomaly module removed |

**Bottom line:** Kerne-ITS-krypto (wire v6, WC-MAC, SSS transitions) **implementeres som SECURITY_LAYERS beskriver**. Audit gaps (doc fiction, legacy surface, stub tests, integration tests) **addressed in 2026-06-16 fix**.

---

## 1. ITS-asymmetric (v0.9 — version-free first release)

### 1.1 Doc ↔ kode matrix

| Dokumenteret (SECURITY_LAYERS / FORMAL_VERIFICATION) | Kode | Status |
|--------------------------------------------------------|------|--------|
| Wire ITS: \(I(M; C_{\text{wire}}) = 0\), ingen key hints | `static_its.rs` | **OK** |
| Per-byte `otm_tag[]`, WC-MAC med skjult `otm_root` | `static_its.rs` + `otm.rs` | **OK** |
| 32-byte `public.key` uden `bind_y`, ingen version-byte | `keypair.rs` | **OK** |
| Lean certificate, ingen `sorry` | `mathematics/Asymmetric/` | **OK** |
| Production `c1=3`, `c2=5`, `c1` invertible | `FieldM31.lean`, `keypair.rs` | **OK** |
| Bundle compositional ITS (wire per chunk + OOB mapping) | `src/bundle/` | **OK** |
| En manifest-profil (OTP-XOR mapping, ingen MANIFEST v1/v2/v3) | `bundle/manifest.rs` | **OK** |
| Wire-only dependency for future SESSION | `default-features = false` | **OK** |

### 1.2 Wire ITS — release-kode vs Eve uendelig CPU

**Mechanisme:** OTP body + Shamir k=2 + morphic blend + WC-MAC (`KEY_DRAW_MASK = 0xFFF`).

| Lag | Status |
|-----|--------|
| Lean ideal model | **Proved** — K8, `fullWireEncShannonIts` |
| Rust release crypto | **Matcher design** |
| Rust adversary tests | **4096²** — `cargo test --release -- --ignored` |

### 1.3 Bundle layer

| Layer | Shannon ITS (Eve uden `secret.key`) | Kode |
|-------|-------------------------------------|------|
| Chunk wire | Ja (per chunk) | `bundle/encrypt.rs` → `static_its::encrypt` |
| Manifest mapping | Epistemisk (OOB `mapping_key`) | `bundle/manifest.rs` |

### 1.4 Source layout (v0.9)

| Path | Role |
|------|------|
| `src/static_its.rs` | Wire core |
| `src/bundle/` | Large-file protocol |
| `src/cli/` | Binary command modules |
| `scripts/verify.sh` | Release tests + Lean + bin build |

CI: `.github/workflows/ci.yml` — `cargo test --all-targets`, `lake build Sacx`.

### 1.5 Død kode (0 call sites, ikke i SECURITY_LAYERS)

| Symbol / fil | Evidence |
|--------------|----------|
| `sss.rs::chain_pad_eval` | 0 call sites; wire v6 bruger ikke link-ratchet |
| `static_its.rs::verify_wire_with_keys` | Eksporteret i `lib.rs`; aldrig kaldt (CLI bruger `verify_wire_bundle`) |
| `static_its.rs::to_wire` | Aldrig kaldt |
| `ciphertext.rs::Ciphertext::to_wire` | Aldrig kaldt |
| `error.rs::AsymmetricError::IoError` | Aldrig konstrueret |
| `morphic.rs::morphic_blend` | Kun egen unit test; inline i `static_its` |
| `masking.rs::encapsulate`, `decapsulate` | Kun `masking.rs` tests |
| `static_its.rs::adversary_v4_mask_recovery` | Stub returnerer `None`; kun legacy tests |
| `static_its.rs::adversary_v6_joint_plaintext_count_2` | Eksporteret; ingen tests/CLI |

### 1.6 Legacy (dokumenteret i SECURITY_LAYERS §3–4 — kan slettes efter doc trim)

| Omfang | Filer/symboler |
|--------|----------------|
| Wire v1–v5 | `parse_v1`–`v5`, `encrypt_v5_wire`, `decrypt_v3`–`v5`, `verify_wire` (v3) |
| Legacy WireCiphertext fields | `key_sum`, `key_slice`, `key_anchor`, `mask_w1/w2`, singular `otm_tag` |
| Hele moduler | `trapdoor_solve.rs` (kun `decrypt_v3`), `masking.rs` (kun via trapdoor_solve) |
| Lean v5 | `WireV5.lean`, `MorphicV5.lean`, `ItsBlindness.lean`, `sacx_v5_certificate` |
| Legacy tests | `v4_public_encrypt_roundtrip`, `multi_sender_v4`, `adversary_v4_*`, `adversary_v5_*` |
| Reference-only | `config.example.toml`, `m61` feature (ingen CI/Lean) |

### 1.7 Bevidst IKKE affald

- v6 `encrypt`/`decrypt`/`verify_wire_bundle`
- `chunked.rs` bundle protocol (MANIFEST v1–v3 er **current**, ikke wire legacy)
- `trapdoor`, `morphic` (unblend), `sss`, `otm`, `field_arith` for v6
- `adversary_v6_trial_byte`, `adversary_v6_byte_plaintext_map` (reelle adversary sim — behold)
- Lean v6 tree + `BundleApp.lean` (Prop defs, ingen theorems — dokumenteret begrænsning)

---

## 2. ITS-OTM + ITS-timelock + SSS_CHAIN

### 2.1 ITS-OTM

| Dokumenteret | Kode | Status |
|--------------|------|--------|
| WC-MAC forgery ≤ 1/p (ITS) | `otm.rs::generate_tag`, `verify_tag` | **OK** |
| SSS forward/backward via sss_chain | `otm.rs` re-exports + `public_attestation.rs` | **OK** |
| Public verify uden secrets | `verify_public_attestation` | **OK** |
| Lean W3 (ekstern ITS repo) | Ikke i denne repos CI | **Delegated** |
| `trapdoor` modul i `lib.rs` | 0 imports i repo | **Død shim** |
| `display_path` i `its_otm.rs` | `#[allow(dead_code)]`, 0 calls | **Død** |

**Tests (11 total):**

| Test | Verificerer | ITS claim? |
|------|-------------|------------|
| `test_otm_generation_and_verification` | Happy path | Delvist |
| `test_chained_tag_with_points` | SSS chain tag | Delvist |
| `test_public_attestation_roundtrip` | Sign/verify | Delvist |
| `test_tampered_bundle_fails` | Message tamper | Nej (ikke tag/share) |
| `test_public_attestation_m31_flow` | M31 flow | Delvist |
| CLI integration (4) | keygen/sign/verify/demo | Operational |
| `bundle_roundtrip_text` (bin) | Text bundle | Operational |

**Gap:** Ingen wrong-tag, wrong-share, eller golden vector T=4578 assertion (math §5).

**Deploy:** `path = "../SSS_CHAIN"` — standalone clone fejler; Docker `COPY . .` uden SSS_CHAIN fejler.

### 2.2 ITS-timelock — self-enclosed ITS via SSS-chaining (korrigeret)

**Vigtig præcisering (mod tidligere forenklet audit):** Timelock er **ikke** "primært computational med et ITS-lag ovenpå". Designet er **self-enclosed ITS** fordi RSW-output \(Y\) **kædes ind** i SSS-algebraen, og SSS transition-kæden fungerer som en **informationsteoretisk one-way** (backward underdetermination — samme mekanisme som `sss_chain` Layer 1), ikke som hash-hardhed.

```text
  .its fil (self-enclosed — ingen escrow)
       |
  RSW: x -> x^(2^T) mod m     [fysisk ur — WHEN Y er kendt]
       |
  Y anker s_{2,0}^m = (Y + m) mod p   [RSW "made ITS through SSS-chaining"]
       |
  T_j = s_{j+1} + s_j  (sss_chain_transition)   [forward-forbrug, backward tvetydig]
       |
  C = M + S_T  (OTP)   +  deny()/time-deny under tvang
       |
  I(M; {T_{1,j}, T_{2,j}, C}) = 0  mod Eve uendelig CPU (coercion-model, math §4)
```

| Rolle | Mekanisme | Mod Eve uendelig CPU | Kode |
|-------|-----------|----------------------|------|
| **Self-enclosed** | Hele puslespillet i `.its` (`x`, `m`, `t`, transitions, `C`, `initial_share_1`) | Ingen online escrow | `SssTimeLock` struct |
| **Fysisk tidsvæg** | RSW sequential squaring | Parallelisering hjælper ikke (DAG dybde \(T\)) | `generate`/`solve` loop L115–119, L214–218 |
| **ITS one-way (SSS chain)** | \(T_j = s_{j+1}+s_j\); backward kan ikke **unikt** bevises uden valg af \(s'_{1,0}\) | **Ja** — underdetermination, ikke faktorisering | `sss_chain_transition` L166–167 |
| **RSW → SSS bro** | \(s_{2,0} = (Y + \text{byte\_idx}) \bmod p\) | Eve med \(Y\) (faktorering) kan stadig ikke bevise sand \(M\) under tvang | L141–144 |
| **Payload ITS** | OTP \(C = M + S_T\); Lagrange \(S_T = 2s_1 - s_2\) | **Ja** i coercion-model | L187–190, `deny()` |
| **Eve faktorerer m instant** | Springer delay — **ikke** ITS-laget | Math §3: "SSS unchanged"; alle \(s'_{1,0}\) giver konsistent \(M'\) | Dokumenteret + Lean |

**One-way her betyder:** SSS-chaining backward **underdetermination** (SSS_CHAIN mat: "not classical OWF — ITS backward ambiguity"). Eve kan algebraisk ikke **bevise** hvilken historie/plaintext der er sand — uanset beregningskraft. RSW er kun **hvornår** \(Y\) bliver tilgængelig; når \(Y\) er kendt, er den **låst ind** i kæden via \(s_{2,0}\), ikke et fritstående bevis-leak.

| Dokumenteret | Kode | Status |
|--------------|------|--------|
| Self-enclosed puzzle | `SssTimeLock` alle felter i én fil | **OK** |
| RSW → SSS anchor | `s2_0 = (Y + idx) % MODULUS` | **OK** |
| SSS transitions (same algebra as sss_chain) | `sss_chain_transition` + `sss_chain_step_forward_from_transition` | **OK** |
| Perfect deniability \(I(M;C)=0\) under tvang | `deny()` + math §4 | **OK i design/Lean**; Rust-test svag |
| RSW delay (parallel-resistent) | Squaring loop | **OK** — rolle = ur, ikke ITS-bevis |
| Lagrange \(S_T = 2s_1 - s_2\) | Inline i `time_lock.rs` | **OK matematisk, duplikat** |
| `its-routing time-deny` decoy `.its` | Kun ROUTING | **Split CLI** — semantik korrekt i ROUTING |
| W4.3 Rust refinement | Lean `stl/` — 0 `sorry` | **Proved i Lean** |
| W4.3 status i SECURITY_LAYERS | "in progress" | **KONFLIKT** med FORMAL_VERIFICATION "Proved + Refined" |
| Golden JSON fixture | `tests/fixtures/m31_section5.json` | **ALDRIG loaded** — værdier hardcodet i test |

**Tests (4 i `time_lock.rs`):**

| Test | Verificerer |
|------|-------------|
| `test_sss_chained_time_lock_roundtrip` | 5-epoch generate/solve |
| `test_perfect_its_deniability` | `denied_msg != message` only |
| `test_transition_publish_forward_inverts` | Manual a+b / t-b (ikke `sss_chain_transition`) |
| `test_golden_m31_section5_fixture` | Hardcoded Y, transition, Lagrange — **ikke JSON** |

**Gap:** Ingen CLI integration tests; RSW golden vector untested; coercion/decoy model untested in Rust.

### 2.3 SSS_CHAIN

| Dokumenteret | Kode | Status |
|--------------|------|--------|
| Field M31 | `sss_chain_field.rs` | **OK** |
| Lagrange k=2 | `sss_chain_lagrange.rs` | **OK** |
| Epoch transitions | `sss_chain_epoch.rs` | **OK** |
| Link generate/validate/arrange | `sss_chain_link.rs` + CLI | **OK** |
| Layer 1 backward underdetermination (ITS) | Math doc | **INGEN Rust property test** |
| "15 tests" i FORMAL_VERIFICATION | 8 `#[test]` i `sss_chain_link.rs` | **DOC FORKERT** |
| Link-chain Lean proofs | "future work" i FORMAL_VERIFICATION | **Ærligt dokumenteret** |

**Test count (verificeret):**

| Module | Tests |
|--------|-------|
| `sss_chain_link.rs` | 8 |
| `sss_chain_epoch.rs` | 2 |
| `sss_chain_field.rs` | 2 |
| `sss_chain_lagrange.rs` | 1 |
| `sss_chain_poly.rs` | 1 |
| `sss_chain_otm.rs` | 1 |
| `tests/cli_integration.rs` | 4 |
| `bin/sss_chain.rs` | 1 |
| **Total** | **20** |

**Deploy:** Self-contained Docker, man, completions — bedst i ecosystem.

---

## 3. ROUTING + ITS-KeyManagement

### 3.1 ROUTING

| Dokumenteret | Kode | Status |
|--------------|------|--------|
| Timelock subcommands | `run_time_lock/unlock/deny` i `main.rs` | **OK** |
| OTM verify in AEH receive | `verify_aeh_otm` → `core_logic::otm::verify_tag` | **OK** |
| Fingerprint erasure pipe | `run_fingerprint_erasure`, send stack | **OK** (ikke testet) |
| `hydra_sss` analog export/import | `main.rs` test | **OK** |
| Anomaly detection / self-healing | `anomaly_detection.rs` | **DOC LØGN** — modul kompileret, **0 call sites** (`grep anomaly_detection::` → none) |
| README §84 "Integration and Network Simulation Tests" | `cargo test` → kun analog SSS + stdio + anomaly self-tests | **DOC LØGN** |
| FORMAL_VERIFICATION: FE strict stack integration test | Ingen FE test i repo | **DOC LØGN** |
| `config.example.toml` | Aldrig læst (defaults hardcoded) | **Doc-reference uden kode** |
| `sss_chain` crate | **Ikke brugt** — `hydra_sss` + timelock crate | **By design** |

**Tests (ROUTING):**

| File | Tests | Production path? |
|------|-------|------------------|
| `main.rs` | 1 (analog SSS) | Partial |
| `stdio.rs` | 1 | Utility |
| `anomaly_detection.rs` | 4 | **Nej — unwired** |

**Dependencies:** Git SSH til upstream crates; path-dep `ITS-fingerprint_erasure` sibling; ingen CI workflow.

### 3.2 ITS-KeyManagement (KeyManagement)

| Dokumenteret | Kode | Status |
|--------------|------|--------|
| Vault ITSKMV2 Argon2id | `vault.rs` | **OK** |
| OTM cert tiers 1a/1b | `otm_cert.rs` | **OK** |
| SHA256 fingerprint deprecated | Stadig i vault/CLI display | **Legacy bevidst** |
| `its-km` shim | `bin/its_id.rs` | **Legacy bevidst** |
| ITS-sessions stub | `orchestrator.rs` log only | **OK (docs siger stub)** |
| Send/receive subprocess | `orchestrator.rs` → `its_asymmetric`, `its-routing` | **OK, untested E2E** |
| Timelock via ROUTING | `validity.rs` subprocess | **OK**; `-c CONFIG` sendes men timelock ignorerer config |
| SSS mod 257 (QR) | `sss_qr.rs` | **OK** — separat field fra M31 routing |
| Wire v6 ITS | Delegeret til assymetric | **OK (docs siger No)** |

**Tests (~18):** vault, sss_qr, validity state machine, otm_cert modes, uhash — **ingen** send/receive/timelock orchestration E2E (OTM QR E2E skipper uden `its_otm`).

**Legacy (dokumenteret deprecated):**

| Item | Path |
|------|------|
| `its-km` binary shim | `src/bin/its_id.rs` |
| `pub mod its_id` re-export | `lib.rs` |
| `contact.rs` alias layer | Re-exports from `key_entry.rs` |
| Fingerprint field + CLI | `key_entry.rs`, `cli.rs` |
| Legacy vault ITSIDV1/PBKDF2 read | `vault.rs` (migrate path) |
| `contacts.example.toml` | Superseded by `entries.example.toml` |

---

## 4. Samlet sletningsliste

### Kolonne A — kan fjernes uden at bryde dokumenteret v6/current path

| Repo | Symbol / fil | Call sites |
|------|--------------|------------|
| assymetric | `chain_pad_eval` | 0 |
| assymetric | `verify_wire_with_keys` | 0 |
| assymetric | `to_wire` (free + `Ciphertext::`) | 0 |
| assymetric | `AsymmetricError::IoError` | 0 |
| assymetric | `morphic_blend` (behold inline i static_its) | 0 prod |
| assymetric | `encapsulate`, `decapsulate` | 0 prod |
| assymetric | Stub: `count_consistent_plaintext_bytes`, `adversary_v6_plaintext_bytes_at` | Vildledende tests |
| OTM | `trapdoor` modul | 0 |
| OTM | `display_path` | 0 |
| ROUTING | **`anomaly_detection.rs` hele fil** + `pub mod` i main | 0 integration |

### Kolonne B — kræver doc-opdatering først (legacy i docs i dag)

| Repo | Omfang | Docs at opdatere |
|------|--------|------------------|
| assymetric | Wire v1–v5 stack (~40% static_its/ciphertext) | SECURITY_LAYERS §3–4, manual, README, FORMAL_VERIFICATION |
| assymetric | `trapdoor_solve.rs`, `masking.rs` | (følger v3 removal) |
| assymetric | Lean v5 certificate tree | Sacx.lean, FORMAL_VERIFICATION |
| assymetric | `config.example.toml` | manual, README |
| assymetric | `m61` feature (alle 6 repos) | manuals |
| ITS-KeyManagement | `its-km`, fingerprint, legacy vault | manual (markerer deprecated) |
| ITS-KeyManagement | `contacts.example.toml`, `[[contact]]` | schema docs |
| ROUTING | `hydra_cli`/`morphic-its` navne | manual, troubleshooting, KEEP_BOUNDARY |
| ROUTING | Anomaly detection docs | README, vision, troubleshooting, KEEP_BOUNDARY |

### Kolonne C — duplikat (konsolider, ikke slet)

| Duplikat | Steder |
|----------|--------|
| `TimeLockText` serialize | `its_timelock.rs` + ROUTING `main.rs` |
| RSW squaring loop ×3 | timelock `generate`/`solve`/`deny` |
| k=2 Lagrange inline | timelock vs `sss_chain::lagrange_interpolate` |
| OTM/timelock `field_arith`/`poly` re-exports | API stability shims |

---

## 5. ITS-gaps rapport

### 5.1 Matematisk uangribelighed mod Eve uendelig CPU

| Claim | Mod Eve | Bevisstyrke | Doc honest? |
|-------|---------|-------------|-------------|
| Wire v6 \(I(M;C)=0\) | **Ja** (hemmeligheder off-channel) | Lean ideal + release crypto; Rust tests under-beviser | **Ja** |
| OTM WC-MAC ≤ 1/p | **Ja** per tag | Lean ekstern; Rust roundtrip only | **Ja** |
| SSS backward ambiguity | **Ja** (algebra) | Math doc; **ingen Rust test** | **Ja** (future work for link Lean) |
| Timelock self-enclosed + SSS one-way + deniability | **Ja** — \(I(M;\text{public puzzle})=0\) under tvang; backward underdetermination | Lean stl/ math §2–4; Rust deny-test svag | **Ja** |
| Timelock RSW som **fysisk ur** (parallel-resistent delay) | Computational timing — **ikke** ITS-bevis, men **ikke** angreb på ITS-laget | By design; math §3 isolerer bypass | **Ja** |
| Timelock: Eve faktorerer \(m\) + kender \(Y\) | Delay bypassed; **ITS deniability uændret** | Math §3 eksplicit | **Ja** |
| Bundle manifest ITS | **Nej** | Docs §7 ærlige | **Ja** |
| ROUTING wire ITS | Delegeret | Ikke egen crypto | **Ja** |
| Vault encryption | **Nej** — Argon2id operational | Docs siger det | **Ja** |

### 5.2 Test vs FORMAL_VERIFICATION gaps

| Claim | Doc says | Rust reality |
|-------|----------|--------------|
| assymetric: 4096² key trials | Lean + FORMAL_VERIFICATION | Unit tests: 256² (`cfg(test)`) |
| assymetric: `adversary_v6_byte_blindness` | Maps to FinEnc OTM joint | Stub constant 256 |
| assymetric: `its_blindness_256_candidates_per_byte` | OTP posterior | Stub constant 256 |
| OTM: T=4578 golden vector | mathematics §5 | Test never asserts 4578 |
| OTM: WC forgery bound | SECURITY_LAYERS | No adversarial test |
| SSS_CHAIN: backward ITS | mathematics | No ambiguity property test |
| SSS_CHAIN: 15 link tests | FORMAL_VERIFICATION | **8 tests** |
| timelock: JSON golden vectors | FORMAL_VERIFICATION | File never loaded |
| timelock: W4.3 | SECURITY_LAYERS "in progress" | FORMAL_VERIFICATION "Proved" |
| ROUTING: FE integration test | FORMAL_VERIFICATION | **No test** |
| ROUTING: timelock/OTM integration | FORMAL_VERIFICATION implied | **No test** |

### 5.3 Lean ↔ Rust semantic gaps

| Gap | Detail |
|-----|--------|
| assymetric morphic c1 | Lean witnesses `c1=1`; Rust `c1=3, c2=5` |
| assymetric K8 definitional | `eveCannotComputeM_v6` bundles structural flags + constant 256 + arithmetic |
| assymetric BundleApp | Prop defs only — no theorems linking to `chunked.rs` |
| OTM Lean | External ITS repo — not in OTM CI |
| timelock W4.3 | Lean `RustModel.lean` rfl proofs; doc status inconsistent |

### 5.4 Doc-løgner og interne konflikter

1. **ROUTING** README/troubleshooting/vision/KEEP_BOUNDARY: anomaly detection + self-healing — **findes ikke i daemon**
2. **ROUTING** README §84: "Integration and Network Simulation Tests" — **findes ikke**
3. **ROUTING** FORMAL_VERIFICATION: FE strict stack integration — **ingen test**
4. **SSS_CHAIN** FORMAL_VERIFICATION: "15 tests" — **8 i link module**
5. **Timelock** SECURITY_LAYERS vs FORMAL_VERIFICATION: W4.3 status konflikt
6. **Timelock** manual: `ITS-routing` navn — repo er **ROUTING** / `its-routing`
7. **Timelock** manual: `deny` vs `time-deny` — forskellig semantik (plaintext vs decoy `.its`)
8. **assymetric** FORMAL_VERIFICATION: stub tests mappet til FinEnc theorems

---

## 6. Svar på plan-spørgsmålene

### Gør koden det dokumentationen har udtænkt?

- **Wire v6, OTM attest, timelock SSS-lag: ja** i release-kode.
- **ROUTING anomaly detection, integration tests, stub adversary "bevis": nej.**
- **Legacy v1–v5: ja** — docs siger de findes; det er bevidst legacy ift. nuværende docs.

### Er den matematisk uangribelig (ITS mod Eve)?

- **Ja** på stier SECURITY_LAYERS markerer ITS — med timelock RSW som **dokumenteret undtagelse**.
- **Bevis-kæden** (Lean↔Rust, tests) er **svagere** end krypto-designet.

### Er der død kode der kan slettes uden at gå ud over dokumentationen?

- **Ja:** Kolonne A (~12 symboler/moduler) — især `chain_pad_eval`, OTM `trapdoor`, ROUTING `anomaly_detection`.
- **Legacy v1–v5:** Kolonne B — kræver doc trim til v6-only først.

---

## 7. Anbefalet rækkefølge (når execution ønskes)

1. **Doc-sandhed** — ret ROUTING anomaly/integration claims, SSS_CHAIN test count, timelock W4.3/deny vs time-deny
2. **Kolonne A sletning** — død kode uden doc-gate
3. **Stub test erstatning** — reelle wire-baserede adversary tests i release profile (4096²)
4. **Kolonne B legacy purge** — v6-only docs + kode + Lean v5 removal
5. **Test gaps** — OTM adversarial, SSS backward ambiguity, timelock JSON + CLI integration
6. **Deploy** — git deps for SSS_CHAIN, monorepo CI, TimeLockText konsolidering

---

*Audit originally generated 2026-06-14. Fixed 2026-06-16 per ecosystem audit fix plan.*
