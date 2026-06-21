# ROUTING — Proof manifest (v9 — ITS-A whitelist + witness consensus + receive gate)

**Formal spec:** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) — authoritative CORE v9: axioms, §Expectations, ValidFwd whitelist, witness consensus, receive gate

**Master cert:** `networkEcosystemCertificateV9` in [`mathematics/MasterTheoremV6.lean`](mathematics/MasterTheoremV6.lean) (= U₈ ∧ `validForwardPartyClosed` ∧ `witnessConsensusClosed` ∧ `forwardReceiveGateClosed`)

**Math gate:** `./scripts/verify_math.sh` — M1–M20, `lake build`, 0 `sorry`, 0 `Prop := True` in `mathematics/`, smoke `ForwardProof.lean` + `ValidForwardParty.lean` + `MasterTheoremV6.lean`  
**Refinement gate (phase 2):** `./scripts/verify_ecosystem.sh` — cargo, pipes M17–M22, Rust refinement  
**Refinement manifest:** [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md) — Lean ↔ Rust map, M17–M22 / X4 / P8.* status

**MathSupremacy:** Eve owns 99.999%+ nodes; all pool/relay HW/SW is backdoored transcript. Security = Lean lemmas only.

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
| L8 | SSS availability | `AvailabilityResilience.lean` | **Operational** | N/A |
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

**Refinement gate:** `cargo test -p its_transport -p its_routing` + `./scripts/verify_ecosystem.sh` (M17–M22)

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
| 1 | Math certificate v5 | `./scripts/verify_math.sh` (M9–M16) |
| 2 | Refinement + product pipes | `./scripts/verify_ecosystem.sh` (M17–M22) |
| 3 | P8.* product DoD | table above |
| 4 | Sibling repos committed at matching tags | `bootstrap.sh` + per-repo `v1.0.0` |
| 5 | Independent review checklist executed | `ITS_INDEPENDENT_REVIEW_CHECKLIST.md` |
| 6 | Meta-tag on all ecosystem repos | `ecosystem-v1.0.0-complete` (git — operator action) |

**Remaining for full ship:** push all repos, tag, execute review checklist on tagged release, public mirror deployment (operational).

**Constitution:** [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md) · [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) v4
