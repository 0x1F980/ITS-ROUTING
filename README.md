# ITS-net: OS Transport Layer & CLI Daemon (its_net_cli)

## GNU General Public License v3.0 Only
Copyright (C) 2026 0x1F464. All rights reserved.

ITS-net is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

---

## 1. Oversigt & Arkitektur

Dette depot udgør **`ITS-net`** (`its_net_cli` craten), transport- og CLI-lag-daemonen i **Morphic Routing Network (ITS/SCPST)** mørkenetssystemet. 

Systemet implementerer det overordnede OS-niveau netværkslag og binder kryptolaget (`ITS`), hardware-abstraktionen (`ITS-hardware`) og konsensus-lagringen (`ITS-ledger`) sammen til en uigennemtrængelig og støj-immun kommunikationstunnel.

### Økosystemets 4-Tier Struktur:
1. **`0x1F464/ITS` (Kryptolag)**: Sterile, `no_std` matematiske formler og krystallografiske felter.
2. **`0x1F464/ITS-net` (Transport/CLI - Dette Depot)**: Netværks-I/O, UDP-courier, aktiv anomali-detektion og CLI-kommandoer.
3. **`0x1F464/ITS-hardware` (HAL)**: Fysiske TRNG-integrationer, seL4-compat, analog share-eksport og sidekanals-blinding.
4. **`0x1F464/ITS-ledger` (Consensus/Vault)**: Sikker offline nøgle-lagring, peer-kontakter (Registry) og AEH blok-synkronisering.

---

## 2. Nøglemekanismer i ITS-net

### 2.1 Aktiv Trafikanalyse-Mitigering & Anomali Detektion
`its_net_cli/src/anomaly_detection.rs`
- **Trafikmønster-Overvågning**: Overvåger løbende indgående og udgående pakkehastigheder, tidsmæssige intervaller og latens.
- **Automatisk Rerouting**: Hvis der detekteres en statistisk afvigelse (anomali) i netværksadfærden (f.eks. Eve forsøger timing-korrelation), afbrydes den eksisterende tunnel øjeblikkeligt, og der rutes asymmetrisk udenom de mistænkte noder.

### 2.2 Transport-Protokol Agnostisk Courier
`its_net_cli/src/main.rs`
- **PacketCourier Trait**: Kommunikationen is helt afkoblet fra det underliggende netværkslag. Den medfølgende UDP-courier sender krypterede SSS-shares, som fremstår som 100% statistisk hvid støj overfor eksterne observatører.

---

## 3. High-Assurance Dokumentations-Suite

Den formelle specifikation og akademiske dokumentation til peer-review og uafhængig audit findes i undermappen `spec/`:

*   **[spec/README.md](spec/README.md)**: Introduktion til specifikations-rammeværket.
*   **[spec/mathematics.md](spec/mathematics.md)**: Formelle matematiske beviser for uigennemtrængelighed og fejlgrænser.
*   **[spec/systems_software.md](spec/systems_software.md)**: Systemsikkerhed, konstant-tid kørsel og compiler-barrierer.
*   **[spec/hardware_sidechannel.md](spec/hardware_sidechannel.md)**: Cryptographic Reverse Firewalls (CRF), Ambient Entropy Harvesting (AEH) og Lorenz-jitter modeller.

For en komplet brugervejledning og dybdegående teoridokumentation, se rod-filerne **`MANUAL.md`** og **`crypto_theory.md`**.

---

## 4. Byggevejledning & Integration

### Byg daemon:
```bash
cargo build --release
```

### Kør testsuiten:
```bash
cargo test --all-features
```
