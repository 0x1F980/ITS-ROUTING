# ITS-routing: Secure Endpoint Doctrine

**Status:** Fase 0 — EP axiom + blast-radius (UES v1.5)  
**Master model:** [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md)

---

## EP axiom (kanal-theorem grænse)

> **Theorem gælder kun** hvis sikker endpoint P ∈ {Alice, Bob} holder K₀ og **ikke** er kompromitteret.

Kompromitteret sikker EP ⇒ nøgler + plaintext eksponeret — **uden for kanal-theorem**. Dette er ikke en implementation-bug; det er en **eksplicit axiom-grænse** i UNATTACKABLE_MODEL.

Lean formaliserer kanal-sikkerhed under antagelsen; EP-sikkerhed er operator-ansvar dokumenteret her.

---

## Blast-radius (kompromittering begrænses)

| Mekanisme | Kilde | Effekt |
|-----------|-------|--------|
| **Ratchet forward secrecy** | `transport_otp_ratchet.rs` | Ét leak ≠ fuld historik |
| **KM dual-password / duress seed** | ITS-KeyManagement | Tvang → decoy ratchet |
| **Timelock deny ridge** | `ridges/timelock.rs` | Deniability ved tvang |
| **Air-gap + precompiled binary** | ITS-routing HEADS_UP | EP uden online Eve |
| **QR / sneakernet** | ridges analog + sneakernet pipe | Ingen online nøgle-exfil |

### Ratchet forward secrecy

`TransportOtpRatchet::step()` avancerer `current` monotont; tidligere epoch-nøgler kan ikke rekonstrueres fra senere state uden seed.

**Lean mirror:** `Transport/RatchetDerivation.lean`

### KM duress

```bash
its-km export-ratchet-seed --out /tmp/seed.bin --password '...' --duress
```

Under tvang giver operatoren decoy-seed; angriber får ikke produktions-ratchet. **Blast-radius:** én session / decoy path — ikke fuld nøglehistorik hvis FS holdes.

### Timelock link

Timelock ridge (`time-lock`, `time-unlock`, `time-deny`) kobler epoch-gate til operatør-kontrolleret release:

- **Deny** path understøtter plausible denial under tvang
- Release timing er epoch-indekseret — se `AEH/EpochGate.lean` (I(S; release) = 0 i kanalscope)

Timelock er **ortogonal ridge** — ændrer ikke pool/AEH algebra, men begrænser EP blast-radius ved fysisk tvang.

---

## Hvad falder ved EP-kompromittering

| Tab | Konsekvens |
|-----|------------|
| K₀ / ratchet-seed | Alle fremtidige epoch-nøgler på den seed |
| Lokalt plaintext | M lækker direkte — **ikke** modelleret i I(S;O)=0 |
| OTM verify disabled | Transcript-brud på EP — design-forbud i CertifiedBuild |

**MathSupremacy:** ondsindet infrastruktur kan **ikke** bryde C/I i O; kompromitteret EP **kan** bryde alt.

---

## Operator-krav (sikker EP)

1. Ratchet-seed kun fra `its-km export-ratchet-seed` — aldrig `demo_aeh_seed` i prod
2. OTM verify obligatorisk før decrypt (release build)
3. L3' konstant harvest — ingen selektiv poll
4. Manuel P↔AEH transition — offline aftale
5. Precompiled binary verificeret; air-gap hvor trussel modellerer online EP-kompromittering
6. Duress-password konfigureret hvor fysisk tvang er i trusselmodel

---

## Relation til O / O⁺

| Observation | EP doctrine |
|-------------|-------------|
| **O** (cellebytes / E) | Theorem: I(S;O)=0 givet korrekt EP |
| **O⁺** (rate/volume) | MetadataSymmetry + L3' — EP skal høste konstant |
| **O⁺** (IP/TLS) | Axiom — EP/netværk uden for theorem |

---

## Cross-references

- [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) — huller H3, master I(S;O)=0
- [ITS-routing_PREBUILD_DOCTRINE.md](ITS-routing_PREBUILD_DOCTRINE.md) — prod AEH flow, demo-forbud
- `mathematics/Adversary.lean` — O vs O⁺ scope
- `mathematics/MathSupremacyDoctrine.lean` — infra vs EP ansvar
