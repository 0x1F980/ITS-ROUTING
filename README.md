# ITS: Pure Mathematical & Cryptographic Core (ITS-crypto)

## GNU General Public License v3.0 Only
Copyright (C) 2026 0x1F464. All rights reserved.

ITS is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

---

## 1. Oversigt & Arkitektur

Dette depot udgør **`ITS-crypto`** (`core_logic` craten), det rene matematiske hjerte i **Hydra-ITS** mørkenetssystemet. For at garantere uafhængighed af operativsystemer og specifikke CPU/hardware-arkitekturer, overholder dette lag følgende strenge krav:
1. **`no_std` Kompatibilitet**: Anvender udelukkende core- og alloc-biblioteker. Har 0% ekstern operativsystem-støj.
2. **Platform- og Enhedsuafhængighed**: Ingen FFI-bindings, rå systemkald eller direkte enhedsadgang.
3. **Konstant-tid kørsel**: Modstår timing- og cache-angreb gennem algebra- og bit-valgs-niveau konstant-tid operationer.
4. **Memory Zeroization**: Aggressiv nulstilling af følsomme mellemliggende værdier og nøglemateriale ved drop.

---

## 2. Matematiske & Kryptografiske Principper

### 2.1 Modular Parameterizable Prime Fields ($\mathbb{Z}_{2^{31}-1}$ / $\mathbb{Z}_{2^{61}-1}$)
`src/field_arith.rs`
- **Mersenne Prime Moduli**: Implementerer lynhurtig pseudo-Mersenne reduktion over de krystallografiske primtal:
  - Standard (m31): $p = 2^{31} - 1$
  - Fremtidssikret (m61): $p = 2^{61} - 1$
- **Unforgeable Integrity**: Ekstremt lav forfalskningssandsynlighed for Wegman-Carter tags ($\epsilon \approx 2^{-31}$ eller $2^{-61}$).

### 2.2 SSS-Chained Perfect Secrecy Trapdoor (SCPST)
`src/tunnel.rs`
- **Onion Cryptography**: Alice (Sender) og Bob (Modtager) etablerer en uigennemtrængelig geometrisk tunnel.
- **Double-Sided SSS Chains**:
  - **Forlæns SSS (Integritet)**: Garanterer pakke-integritet trin for trin.
  - **Baglæns SSS (Autoritet)**: Verificerer afsenderens autoritet og modvirker replay-angreb i konstant-tid.
- **Wegman-Carter OTMs**: Giver informations-teoretisk sikker dataintegritet.

### 2.3 Morphic Network Coding (MNC)
`src/morphic_proof.rs`
- **Blind Linear Mixing**: Mix-noder på ruten kan udføre blinde lineære operationer (addere eller flette shares) direkte på de krypterede pakker uden nogensinde at have adgang til dekrypteret data eller de underliggende Shamir-polynomier.
- **Anti-Traffic Analysis**: Neutraliserer passiv trafik- og tidsanalyse-overvågning (Eve kan ikke spore en pakke, da dens algebraiske form ændres morphisk ved hvert hop).

### 2.4 State Ratchets & Time-Locks
`src/ratchet.rs` & `src/time_lock.rs`
- **Dual-Seed Duress Ratchet**: Hver node opretholder en dynamisk tilstand, der ændres ved hver afsendt/modtaget pakke. Understøtter duress-nøgler, der afspiller plausible falske profiler ved tvungen overlevering.
- **Deniable Time-Lock**: SSS-chained tidslåse der tvinger modtageren til at udføre en række algebraisk uundgåelige, sekventielle modulære kvadreringer, hvilket forhindrer paralleliseret dekryptering på supercomputere eller kvante-computere.

---

## 3. Byggevejledning & Integration

### Byg crate (`no_std`):
```bash
cargo build --release
```

### Kør testsuiten:
```bash
cargo test --release
```
