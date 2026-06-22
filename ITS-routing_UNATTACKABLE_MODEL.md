# ITS-routing: Unattackable Model (v9 — 100% ITS C · I · A)

**Formal spec (formulas + Lean map):** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md)

**Status:** Formal math certificate v9 — software refinement explicitly phase 2  
**Master theorem:** `mathematics/MasterTheoremV6.lean` — `networkEcosystemCertificateV9` (M1–M20)  
**Absolute deniability:** `mathematics/PlausibleDeniabilityAbsolute.lean`  
**Verify:** `./scripts/verify_math.sh` — M1–M20, `lake build`, 0 `sorry`, smoke certificate  
**Refinement (later):** `./scripts/verify_ecosystem.sh` — cargo + E2E pipes

---

## Uangribeligheds-lov (MathSupremacy)

> **Matematik er den eneste sikkerhedskilde.** Eve ejer 99.999%+ noder og al pool/relay
> software/hardware (backdoored). Det er **transcript** — ikke sikkerhed.  
> **O** = globale celler uden provenance. **IP_obs** lukket under BIS (B1–B3).  
> Alice **eller** Bob kører math-trusted executor. Eve lærer **0 bits** om hvem der sendte,
> hvem der modtog, og hvilken vej beskeden tog — i **O** og **IP_obs**.

\[
\boxed{I(S;\, O_{\text{Eve}}) = 0 \;\wedge\; I(\text{author};\, O) = 0 \;\wedge\; I(\text{recipient};\, O) = 0}
\]
\[
\boxed{I(\text{author};\, \text{IP\_obs}) = 0 \;\wedge\; I(\text{recipient};\, \text{IP\_obs}) = 0 \;\wedge\; I(\text{flow};\, \text{IP\_obs}) = 0}
\]

**S** = (M, ratchet, link, label, timing-secret) — hele S, ikke kun M.

---

## Observation alphabet

| Symbol | Indhold | Master theorem? | Lean |
|--------|---------|-----------------|------|
| **O** | Cellebytes, benign E-observation | **Ja** | `ObservationAlphabet.lean` |
| **O⁺** | Rate, volume, participation pattern | Separate lemmaer | `MetadataSymmetry.lean`, `OplusClosure.lean` |
| **O_phys / IP_obs** | src/dst/shape (transport) | **Ja under BIS** | `IPObservation.lean`, `BroadcastIPSymmetry.lean` |

---

## Author vs recipient vs IP (v4)

| Variabel | Claim | Status |
|----------|-------|--------|
| **I(author; O)** | Ingen forfatter i kanal-O | **Theorem** — `AuthorAttributionZero.lean` |
| **I(recipient; O)** | Ingen modtager i kanal-O headers | **Theorem** — `RecipientAttributionZero.lean` |
| **I(flow; O), I(flow; IP)** | Ingen path-korrelation | **Theorem** — `FlowAttributionZero.lean` |
| **I(author; IP_obs), I(recipient; IP_obs)** | Ingen IP-afsender/modtager | **Theorem under BIS** — `BroadcastIPSymmetry.lean`; B2 **derived** — `BroadcastIPDerivation.lean` |
| **Timeless C/I (P6.*)** | Compute-epoch uafhængig | **Theorem** — `TimelessSecurity.lean` |
| **Medium independence (P2.3)** | Wire-seal på pool/AEH/offline | **Theorem** — `MediumIndependence.lean` |
| **M10 ecosystem cert v5** | C1∧C2∧C3∧C4∧T∧timeless∧medium | **Theorem** — `MasterTheorem.lean` (C4 Stl import) |
| **C4 timelock (P5.*)** | Coercion + RSW aux + master compose | **Theorem** — `CoercionModel.lean`, `Transport/TimelockComposition.lean` |
| **Ingen skyldig node** | Alle plausibelt benægtelige | **Theorem** — `PlausibleDeniabilityAbsolute.noGuiltyNode` |
| **Either EP** | Alice encryptor **∨** Bob verify-oracle | **Theorem** — `EndpointEitherOr.lean` |
| **Sybil 99.999%+** | 0 ekstra bits om M | **Proved (finite-MI)** — `SybilDoctrine.lean` |

> **Sprint 1 MI:** C3 (`UnifiedEpochStream`), Sybil, and L3' (`MetadataSymmetry`) are **Proved (finite-MI)** via `FiniteMutualInfo.lean`. See [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) §XII and [PROOF_MANIFEST.md](PROOF_MANIFEST.md).

Author/recipient/flow lukkes via pool (ingen provenance), AEH (φ ~ 𝒟_benign), broadcast-forward,
SSS multi-IP courier, og BIS — **ikke** via at stole på Eve's nodes.

---

## Huller-lukning matrix (lemma-ID — ikke `.rs`)

| Huller | Angreb | Lukning | Lean |
|--------|--------|---------|------|
| **H1** | Availability — censur, selektiv sletning | SSS, multi-courier, AEH, sneakernet | `AvailabilityResilience.lean`, `OfflineChannel.lean` |
| **H2** | O⁺ metadata — rate/volume/timing | L3' konstant harvest, fast størrelse | `MetadataSymmetry.lean`, `OplusClosure.lean` |
| **H3** | Endpoint-kompromittering | EP-split: encryptor vs verify-oracle | `EndpointSplit.lean` — `secureEndpointAxiom` udenfor kanal |
| **H4** | Forkert implementation | **Phase 2** refinement | `Refinement/EpochCellCorrectness.lean` (ikke i master-cert) |

---

## I(S; O) = 0 — lemma-kæde

| # | Lemma | Mode | Lean module | Math status |
|---|-------|------|-------------|-------------|
| L1 | Wire Shannon + cell indistinguishability | begge | `Transport/WireComposition.lean` | **Proved** (asymmetric import) |
| L2 | OTM WC-MAC, P(forge) ≤ 1/p | begge | `IntegrityAxiom.lean` | **Axiom** |
| L3 | C_e ~ 𝒟, altid emit | P | `UnifiedEpochStream.lean` | **Proved (finite-MI)** |
| L4 | φ ~ 𝒟_benign | AEH | `AEH/StegoIndistinguishability.lean` | **Proved** |
| L5 | I(S; release) = 0 | begge | `AEH/EpochGate.lean` | **Proved** |
| L6 | I(link; O) = 0 under L3+L3' | P | `LinkParticipation.lean` | **Proved** |
| L7 | φ ~ 𝒟_benign ⇒ link-blind i AEH | AEH | `PlausibleDeniability.lean` | **Proved** |
| L8 | SSS rekonstruktion under ≤ f deletion | A | `AvailabilityResilience.lean` | **Proved (v9 ITS-A component)** |
| L9 | Composition end-to-end | begge | `Transport/Composition.lean` → `UnattackableCertificate.lean` | **Proved** |
| L10 | I(link; O⁺_{rate,volume}) = 0 | begge | `MetadataSymmetry.lean` | **Proved (finite-MI)** |
| L11 | CoverTransport ⇒ konstant O⁺ deltagelse | P | `ParticipationSymmetry.lean` | **Postulate-under-P1–P3** |
| L12 | I(link; O⁺_participation) = 0 | P | `OplusClosure.lean` | **Postulate-under-P1–P3** |
| L13 | Passiv ISP-inference ⊆ aktiv Eve | begge | `ComparativeThreatDoctrine.lean` | **Proved** |

**Master:** `UnattackableCertificate.lean` — C1–C4 + attribution + O⁺ + offline + EP-split  
**Participation:** `ParticipationTheorem.lean` — I(author;O)=0, ingen provenance i O  
**Broadcast:** `BroadcastForward.lean` — hop forward bevarer I(author;O)=0  
**Doctrine:** [ITS-routing_PARTICIPATION_SYMMETRY.md](ITS-routing_PARTICIPATION_SYMMETRY.md) — P1–P3 operator postulates

---

## Endpoint split (H3)

| Lemma | Antagelse | Konklusion | Lean |
|-------|-----------|------------|------|
| `wireConfidentiality` | Sikker **encryptor** | I(M; O) = 0 | `EndpointSplit.lean` |
| `wireIntegrity` | Sikker **verify-oracle** | P(forge) ≤ 1/p | `EndpointSplit.lean` |
| `endToEndChannel` | Begge + composition | Full CIA i kanal | `EndpointSplit.lean` |

EP-kompromittering = **axiom udenfor kanal** (`secureEndpointAxiom`) — ikke software-dokument i theorem-kæden.

---

## Modes P / AEH

| Mode | O | Courier | Transition |
|------|---|---------|------------|
| **P — UES Pool** | (C₁,…,C_T) pool bytes | PoolCourier | Manuel — **ingen auto-switch** |
| **AEH — last-resort** | obs(E_i) benign masse | AehCarrier embed φ | Manuel ved pool-ban |

Begge modes deler **samme** `step` + wire + OTM — kun courier/embed ændres.

---

## CIA triaden

| Pille | ITS-rang | Lean |
|-------|----------|------|
| **C — Confidentiality** | **100% ITS** — \(I(S;O)=0\), Sybil irrelevant, wire Shannon | `CIA_Doctrine.lean`, C1 via `WireComposition.lean`, `SybilDoctrine.lean` |
| **I — Integrity** | **100% ITS** — \(P(\text{forge})\le 1/p\); verify on A2′ EP | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` |
| **A — Availability** | **100% ITS (v9)** — ProofFwd, \(\mathcal{M}_{\text{valid}}\), witness k-of-n, ReceiveGate, SSS | `CIA_Doctrine.availabilityITSForward`, `ForwardProof.lean`, `ValidForwardParty.lean`, `WitnessConsensus.lean`, `ForwardReceiveGate.lean`, `AvailabilityResilience.lean` |

**Outside (explicit):** both EP compromised; side-channels; \(O_{\text{net}}=\emptyset\); no A2′ witness + empty \(\mathcal{M}_{\text{valid}}\).

**Honest limit:** A ≠ Shannon “always delivers”; A = log-proof + whitelist + reroute when valid mirrors/witness exist.

---

## Sybil

Eve ejer alle noder undtagen én sikker endpoint. Sybil fake-posters afvises af OTM eller er chaff fra 𝒟.

\[
I(M;\, O_{\mathcal{A}}) = 0 \quad \text{(Sybil-strategier } \mathcal{A} \text{)}
\]

**Lean:** `SybilDoctrine.lean` — Sybil irrelevant for C/I; kun A og O⁺. **Sprint 1:** Shannon claim **Proved (finite-MI)** via `FiniteMutualInfo.lean`.

---

## MathSupremacy

> Matematik er den eneste sikkerhedskilde. Eve's ondsindet pool-SW/HW er **transcript** — ikke sikkerhedsbærer.

| Lag | Rolle |
|-----|-------|
| Lean lemmaer | Sandhedskilde |
| step, wire, OTM | Algebra |
| Eve infrastruktur | Kan forsinke/slette/forge → **A**, ikke C/I |
| Sikker EP (Alice/Bob) | Enkel trusted executor |

**Lean:** `MathSupremacyDoctrine.lean`

---

## Adversary scope

| I O (theorem) | Ude af O (axiom / operator) |
|---------------|-------------------------------|
| Pool-cellebytes | IP/MAC/TLS ved pool-HTTP |
| AEH: benign E-observation | Fysisk sidekanal |
| Konstant epoch-rate i kanalbytes | EP-kompromittering |
| | Selektiv harvest uden L3' |
| | Auto-switch P↔AEH |

**Lean:** `Adversary.lean` — channel MI zero **conditional** on wire certificate (`Transport/WireComposition.lean`).

---

## Offline / sneakernet

Når \(O_{\text{net}} = \emptyset\): I(S; O_net) = 0 trivialt. Security reducerer til wire algebra på offline medium + verify-oracle på Bob.

**Lean:** `OfflineChannel.lean`

---

## Size-independence (N = 1)

Anonymity set = 𝒟 (P) eller alle E-konsumenter (AEH) — **ikke** ITS-brugerantal.

**Lean:** `FewUserDoctrine.lean`

---

## Forbud (prod)

- Isoleret lillebruger-ITS-net med identificerbar provenance
- Punkt-til-punkt routing i O
- Auto-switch P↔AEH
- Computational crypto på hot path
- `WIKI_STEGO:` demo-strenge i release
- `--timeout-secs` selektiv receive (prod)

---

## Lean manifest (v9 math certificate)

```
mathematics/
├── ObservationAlphabet.lean    # O, O⁺, IP_obs scope (v4)
├── IPObservation.lean          # IP_obs structure, Eve 99.999%+ Sybil
├── BroadcastIPSymmetry.lean    # B1–B3 → I(author/recipient; IP)=0
├── RecipientAttributionZero.lean
├── FlowAttributionZero.lean
├── EndpointEitherOr.lean       # Alice ∨ Bob math-trusted EP
├── SSSMultiIPCourier.lean      # shares from many IPs, all blind
├── PlausibleDeniabilityAbsolute.lean  # master absolute deniability
├── EndpointSplit.lean          # encryptor vs verify-oracle
├── IntegrityAxiom.lean         # C2 OTM axiom (import-ready)
├── Adversary.lean              # channel MI (conditional on wire)
├── BroadcastForward.lean
├── OplusClosure.lean           # P1–P3 → O⁺ closure
├── OfflineChannel.lean         # sneakernet / blackout
├── AuthorAttributionZero.lean  # I(author;O)=0 package
├── UnattackableCertificate.lean # M7 master theorem
├── UnifiedEpochStream.lean
├── LinkParticipation.lean
├── ParticipationTheorem.lean
├── PlausibleDeniability.lean
├── AvailabilityResilience.lean   # SSS bound (v9 ITS-A component)
├── MetadataSymmetry.lean
├── ForwardProof.lean             # ProofFwd + alternateRoute (v9 ITS-A)
├── ValidForwardParty.lean        # ValidFwd whitelist + de-whitelist
├── WitnessConsensus.lean         # k-of-n witness consensus
├── ForwardReceiveGate.lean       # receiveGate on M_valid
├── MasterTheoremV6.lean          # networkEcosystemCertificateV9
├── ParticipationSymmetry.lean
├── ComparativeThreatDoctrine.lean
├── CIA_Doctrine.lean
├── SybilDoctrine.lean
├── MathSupremacyDoctrine.lean
├── FewUserDoctrine.lean
├── Transport/
│   ├── WireComposition.lean    # C1 asymmetric Shannon chain
│   ├── Composition.lean        # L9 → master cert
│   └── ...
├── AEH/
└── Refinement/                 # phase 2 — not in master cert
    └── EpochCellCorrectness.lean
```

**Cross-repo:** `ITS-asymmetric/mathematics` — `require «asymmetric-math»` in `lakefile.lean`

**Verify (math only):**

```bash
./scripts/verify_math.sh
```

**Win-conditions M1–M20:** see [PROOF_MANIFEST.md](PROOF_MANIFEST.md) v9
