# ROUTING — Proof manifest (v10 — implementation refinement + v9 ideal)

**Formal spec:** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) — authoritative CORE v10: axioms, §Expectations, ValidFwd whitelist, witness consensus, receive gate, refinement

**Master cert (ideal):** `networkEcosystemCertificateV9` in [`mathematics/MasterTheoremV6.lean`](mathematics/MasterTheoremV6.lean) (= U₈ ∧ ITS-A bundle)

**Master cert (implementation):** `networkImplementationCertificateV10` in [`mathematics/MasterTheoremV6.lean`](mathematics/MasterTheoremV6.lean) (= v9 ∧ refinement closed bundles — epoch cell, ValidFwd, witness, receive gate, client pool)

**Math gate:** `./scripts/verify_math.sh` — M1–M26, `lake build`, 0 `sorry`, 0 `Prop := True` in `mathematics/`, smoke refinement + v10 cert  
**Refinement gate (theorem):** M23–M26 in `verify_math.sh`  
**Ecosystem gate (smoke):** `./scripts/verify_ecosystem.sh` — cargo, pipes M17–M22 (M21–M22 smoke only after v10)  
**Refinement manifest:** [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md) — Lean ↔ Rust map, M23–M26 / v10.1 sibling tracks

**MathSupremacy:** Eve owns 99.999%+ nodes; all pool/relay HW/SW is backdoored transcript. Security = Lean lemmas only.

---

## CIA pillars — narrative + example refs (v9)

Each pillar is **proved** under A0–A2′. Concrete numeric walkthroughs live in [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) §Va and [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) (Eve scenario).

**C — Confidentiality.** Eve sees all of \(O\) including \(10^9\) Sybil injections; finite-MI + Shannon wire give \(I(M;O)=0\) bits for a 256-bit message (uniform posterior). Sybil adds zero information (`SybilDoctrine.lean`, `FiniteMutualInfo.lean`). *Example:* §Va C-table — 256-bit \(H(M|O)=H(M)\).

**I — Integrity.** OTM WC-MAC over \(\mathbb{F}_p\), \(p=2147483647\): \(P(\text{forge})\leq 1/p\). Eve's \(10^{12}\) forgery attempts yield \(\leq 465\) expected accepts; verify runs only on A2′ EP (`IntegrityAxiom.lean`). *Example:* §Va I-table — \(N/p\) bound.

**A — Availability (ITS-A v9).** Not Shannon delivery — log-proof + \(\mathcal{M}_{\text{valid}}\) whitelist + de-whitelist on omit + witness k-of-n (\(k=2, n=3\)). One honest forwarder in \(\mathcal{M}_{\text{valid}}\) suffices against \(10^9\) Eve nodes. *Example:* §Va A-table — epochs 0–5, Eve-A omit @3, `omit_de_whitelists_mirror`, `ProofFwd` via Charlie. **Outside:** empty \(\mathcal{M}_{\text{valid}}\), no witness.

---

## Master math certificate (v4)

| Claim | Lean module | Math status | v4 MI status |
|-------|-------------|-------------|--------------|
| **M7 — unattackable certificate v4** | `UnattackableCertificate.lean` | **Proved** | N/A (certificate shell) |
| **C4 absolute plausible deniability** | `PlausibleDeniabilityAbsolute.lean` | **Proved** | N/A |
| C1 wire Shannon \(I(M;O)=0\) | `Transport/WireComposition.lean` → `Asymmetric.fullWireEncShannonIts` | **Proved** (cross-repo) | **Proved** (import) |
| C2 integrity P(forge) ≤ 1/p | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` | **Proved** (cross-repo) | **Proved** (import) |
| C3 stream + Sybil + MathSupremacy | `UnifiedEpochStream.lean`, `SybilDoctrine.lean`, `MathSupremacyDoctrine.lean` | **Theorem** | **Proved** (finite-MI) |
| I(author; O) = 0 | `AuthorAttributionZero.lean` | **Proved** | **Proved** |
| I(recipient; O) = 0 | `RecipientAttributionZero.lean` | **Proved** | **Proved** |
| I(flow; O) = 0, I(flow; IP) = 0 | `FlowAttributionZero.lean` | **Proved** | **Proved** |
| I(author; IP_obs) = 0, I(recipient; IP_obs) = 0 | `BroadcastIPSymmetry.lean` (B1–B3) | **Theorem under BIS** | **B2 derived** (`BroadcastIPDerivation.lean`) |
| B2 from L3 + cell | `BroadcastIPDerivation.lean` | **Proved** | **Proved** |
| Timeless C/I (P6.*) | `TimelessSecurity.lean` | **Proved** | **Proved** |
| Medium independence (P2.3) | `MediumIndependence.lean` | **Proved** | **Proved** |
| **M10 — networkEcosystemCertificateV5** | `MasterTheorem.lean` | **Proved** (C4 Stl import) | N/A |
| **M17 — networkEcosystemCertificateV6** | `MasterTheoremV6.lean` | **Proved** (Absolut A + BIS full + roles) | N/A |
| **M17+ — networkEcosystemCertificateV7** | `MasterTheoremV6.lean` | **Proved** (zero `Prop := True`; P1–P3 derived) | N/A |
| **M19 — ITS-A forward proof** | `ForwardProof.lean` | **Proved** (ProofFwd + alternateRoute) | N/A |
| **M19+ — networkEcosystemCertificateV8** | `MasterTheoremV6.lean` | **Proved** (U₇ ∧ aItsForwardProofClosed) | N/A |
| **M20 — ValidFwd whitelist** | `ValidForwardParty.lean` | **Proved** (omit ⇒ de-whitelist) | N/A |
| **M20 — witness k-of-n consensus** | `WitnessConsensus.lean` | **Proved** (consensus ⇒ ProofFwd) | N/A |
| **M20 — forward-receive gate** | `ForwardReceiveGate.lean` | **Proved** (M_valid alternate path) | N/A |
| **M20+ — networkEcosystemCertificateV9** | `MasterTheoremV6.lean` | **Proved** (U₈ ∧ v9 ITS-A bundle) | N/A |
| **Phase 3 — networkImplementationCertificateV10** | `MasterTheoremV6.lean` + `Refinement/*.lean` | **Proved** (v9 ∧ refinement bundles) | N/A |
| ValidFwd refinement | `Refinement/ValidForwardRefinement.lean` | **Proved** | N/A |
| Witness consensus refinement | `Refinement/WitnessConsensusRefinement.lean` | **Proved** | N/A |
| Forward receive gate refinement | `Refinement/ForwardReceiveGateRefinement.lean` | **Proved** | N/A |
| Client pool refinement | `Refinement/ClientPoolRefinement.lean` | **Proved** | N/A |
| Epoch cell refinement closed | `Refinement/EpochCellCorrectness.lean` | **Proved** (counter + support) | N/A |
| P1–P3 participation postulates | `OplusClosure.participationPostulatesDerived` | **Proved** (L3 + pool + L3') | **Proved** |
| B1+B3 from L3+pool+P1–P3 | `BroadcastIPDerivation.bisFullyDerived` | **Proved** | **Proved** |
| Absolut A + forward proof | `CensorshipDisclosure.lean`, `ForwardProof.lean` | **Proved** (v8) | N/A |
| Public pool multicast | `PublicPoolMulticast.lean` | **Proved** | N/A |
| Role-aware noGuiltyNode | `RoleAwareDeniability.lean` | **Proved** | **Proved** |
| **C4 timelock deniability** | `CoercionModel.lean`, `Transport/TimelockComposition.lean` → `Stl.Security.Deniability` | **Proved** (cross-repo) | N/A |
| SSS multi-IP courier | `SSSMultiIPCourier.lean` | **Proved** | **Proved** |
| Either EP secure (Alice ∨ Bob) | `EndpointEitherOr.lean` | **Proved** | **Proved** |
| No guilty node (all deniable) | `PlausibleDeniabilityAbsolute.noGuiltyNode` | **Proved** | **Proved** |
| O⁺ closure L10–L12 under P1–P3 | `OplusClosure.lean` | **Proved** (P1–P3 derived) | **Proved** |
| Offline / sneakernet O_net = ∅ | `OfflineChannel.lean` | **Proved** | **Proved** |
| L9 mode composition P ⊗ AEH | `Transport/Composition.lean` | **Proved** | **Proved** |
| L13 comparative threat | `ComparativeThreatDoctrine.lean` | **Proved** | **Proved** |
| A availability (ITS forward proof + v9 gate) | `ForwardProof.lean`, `ValidForwardParty.lean`, `WitnessConsensus.lean`, `ForwardReceiveGate.lean` | **Proved** (v9) | N/A |

---

## Observation alphabet (theorem scope v4)

| Symbol | Meaning | Lean | In master theorem? |
|--------|---------|------|--------------------|
| **O** | Channel bytes / benign E-observation | `ObservationAlphabet.lean` | Yes |
| **O⁺** | Rate, volume, participation | `MetadataSymmetry.lean`, `OplusClosure.lean` | Under P1–P3 |
| **IP_obs** | src/dst/shape tuples | `IPObservation.lean`, `BroadcastIPSymmetry.lean` | **Yes under BIS** |
| Side-channel (non-IP) | Timing power EM | — | Operator / EP axiom |

---

## Lemma chain (math-only — no `.rs` in proof path)

| # | Lemma | Lean module | Math status | v4 MI status |
|---|-------|-------------|-------------|--------------|
| L1 | Wire + cell indistinguishability | `Transport/WireComposition.lean`, `Transport/Cell.lean` | **Proved** | **Proved** |
| L2 | OTM WC-MAC floor | `Transport/Field.lean`, `IntegrityAxiom.lean` → `Otm.OtmIntegrity` | **Proved** (cross-repo) | **Proved** (import) |
| L3 | C_e ~ 𝒟 constant emit | `UnifiedEpochStream.lean` | **Theorem** | **Proved** (finite-MI) |
| L4 | φ ~ 𝒟_benign | `AEH/StegoIndistinguishability.lean` | **Proved** | **Proved** |
| L5 | I(S; release) = 0 | `AEH/EpochGate.lean` | **Proved** | **Proved** |
| L6 | I(link; O) = 0 | `LinkParticipation.lean` | **Proved** | **Proved** |
| L7 | AEH link-blind | `PlausibleDeniability.lean` | **Proved** | **Proved** |
| L8 | SSS availability | `AvailabilityResilience.lean` | **Proved (SSS bound; subsumed in v9 ITS-A bundle)** | N/A |
| L9 | Composition | `Transport/Composition.lean` | **Proved** | **Proved** |
| L10 | I(link; O⁺_{rate,volume}) = 0 | `MetadataSymmetry.lean` | **Theorem** | **Proved** (finite-MI) |
| L11 | CoverTransport constant O⁺ | `ParticipationSymmetry.lean` | **Proved** (L3 + L3') | **Proved** |
| L12 | I(link; O⁺_participation) = 0 | `ParticipationSymmetry.lean`, `OplusClosure.lean` | **Proved** (P3 + finite-MI) | **Proved** |
| L13 | Passive ISP ⊆ active Eve | `ComparativeThreatDoctrine.lean` | **Proved** | **Proved** |
| — | N=1 size-independent I(M;O)=0 | `FewUserDoctrine.lean` | **Proved** | **Proved** (finite-MI) |
| — | Broadcast forward I(author;h(O))=0 | `BroadcastForward.lean` | **Proved** | **Proved** (finite-MI) |
| — | I(author; O) package | `AuthorAttributionZero.lean` | **Proved** | **Proved** |
| — | I(recipient; O) and IP | `RecipientAttributionZero.lean` | **Proved** | **Proved** |
| — | I(flow; O) and I(flow; IP) | `FlowAttributionZero.lean` | **Proved** | **Proved** |
| — | BIS B1–B3 IP symmetry | `BroadcastIPSymmetry.lean` | **Theorem under postulates** | **B2 derived** |
| — | B2 from L3 + cell | `BroadcastIPDerivation.lean` | **Proved** | **Proved** |
| — | Timeless security P6.* | `TimelessSecurity.lean` | **Proved** | **Proved** |
| — | Medium independence | `MediumIndependence.lean` | **Proved** | **Proved** |
| — | Master v5 ecosystem cert | `MasterTheorem.lean` | **Proved** (C4 Stl import) | N/A |
| — | C4 coercion model (P5.1) | `CoercionModel.lean` → `Stl.Security.Deniability` | **Proved** (cross-repo) | N/A |
| — | Timelock composition (P5.2–P5.3) | `Transport/TimelockComposition.lean` | **Proved** | N/A |
| — | SSS multi-IP courier | `SSSMultiIPCourier.lean` | **Proved** | **Proved** |
| — | Either EP (Alice ∨ Bob) | `EndpointEitherOr.lean` | **Proved** | **Proved** |
| — | Absolute deniability master | `PlausibleDeniabilityAbsolute.lean` | **Proved** | **Proved** |

**Cross-repo C1 source:** `ITS-asymmetric/mathematics` — `Asymmetric.Shannon`, `Asymmetric.WireAdversary`

---

## Refinement manifest (phase 2 — software/hardware)

Full map: [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md)

| Concern | Implementation | Refinement status |
|---------|----------------|-------------------|
| OTP ratchet forward | `its_transport::transport_otp_ratchet` | **Proved** — `Transport/RatchetDerivation.lean` + `rust_ratchet_algebra_matches_lean` |
| UES epoch cell / `step` | `its_transport::epoch_cell` | **Proved (counter + support)** — `Refinement/EpochCellCorrectness.lean`; `rust_epoch_cell_refines_ideal` |
| End-to-end binary | `client.rs` pool/AEH paths | E2E pipes |
| OTM verify | `aeh.rs`, `epoch_cell.rs` | `ITS-OTM` (external) |
| AEH φ embed | `aeh_carrier.rs` | Stego pipe |
| Pool publish | anonym pool HTTP | `pipe_its_pool_e2e.sh` |
| CoverTransport | cover harvest | `pipe_its_cover_harvest_e2e.sh` |
| Sneakernet | offline courier | `pipe_its_sneakernet_e2e.sh` |
| SOCKS egress | app proxy | `pipe_its_socks_pool_e2e.sh` |
| KM send/receive | operator glue | `pipe_its_km_pool_e2e.sh` |
| Timelock | C4 ridge | `pipe_timelock.sh` |
| Public mirror | reference deploy | `pipe_its_http_pool_e2e.sh` |
| ValidFwd / M_valid | `ValidForwardParty.lean` | `its_routing::valid_forward_party` | **Proved** — `Refinement/ValidForwardRefinement.lean` |
| Witness consensus | `WitnessConsensus.lean` | `its_routing::witness_consensus` | **Proved** — `Refinement/WitnessConsensusRefinement.lean` |
| Forward receive gate | `ForwardReceiveGate.lean` | `receive_gate`, `WhitelistMultiCourier` | **Proved** — `Refinement/ForwardReceiveGateRefinement.lean` |
| Pool client path | `ForwardReceiveGate.harvestPermitted` | `courier.rs` | **Proved** — `Refinement/ClientPoolRefinement.lean` |

**Refinement gate (theorem):** `./scripts/verify_math.sh` M23–M26 + `cargo test -p its_routing valid_forward consensus`  
**E2E smoke:** `./scripts/verify_ecosystem.sh` M21–M22 (not primary proof after v10)

---

## Product DoD (P8.* — Sprint 5)

| Postulate | Claim | Gate | Status |
|-----------|-------|------|--------|
| **P8.1** | Anonym file/message via pool default | `pipe_its_pool_e2e.sh` | **Green** |
| **P8.2** | App egress via SOCKS pool proxy | `pipe_its_socks_pool_e2e.sh` + D30 | **Green** |
| **P8.3** | Censur: fountain + multi-mirror + AEH + sneakernet | M21 pipes | **Green** |
| **P8.4** | One-command send via `its-km` | `pipe_its_km_pool_e2e.sh` | **Green** |
| **P8.5** | Timelock generate/unlock/deny | `pipe_timelock.sh` | **Green** |
| **P8.6** | Public pool deploy BIS/P1–P3 | M18 + D9 | **Green** |
| **P8.7** | Migration Tor/I2P/Nym → ITS | D7 + `ITS_MIGRATION_GUIDES.md` | **Green** (local verify) |

---

## Ecosystem tag criteria (`ecosystem-v1.0.0-complete` — Sprint 6 prep)

**Do not tag without user confirmation.** When all criteria are green:

| # | Criterion | Gate |
|---|-----------|------|
| 1 | Math certificate v10 | `./scripts/verify_math.sh` (M1–M26) |
| 2 | Ecosystem smoke (pipes) | `./scripts/verify_ecosystem.sh` (M17, M21–M22 smoke) |
| 3 | P8.* product DoD | table above |
| 4 | Sibling repos committed at matching tags | `bootstrap.sh` + per-repo `v1.0.0` |
| 5 | Independent review checklist executed | `ITS_INDEPENDENT_REVIEW_CHECKLIST.md` |
| 6 | Meta-tag on all ecosystem repos | `ecosystem-v1.0.0-complete` (git — operator action) |

**Remaining for full ship:** push all repos, tag, execute review checklist on tagged release, public mirror deployment (operational).

**Constitution:** [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md) · [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) v9
