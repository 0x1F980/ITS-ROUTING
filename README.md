# ITS-net: OS Transport Layer & CLI Daemon (Transport/CLI)

## GNU General Public License v3.0 Only
Copyright (C) 2026 0x1F464. All rights reserved.

ITS-net is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

---

## 1. Overview & Architecture

`ITS-net` (implementing the `its_net_cli` binary, formerly `hydra_cli`) is the system-level daemon and transport engine of the **Morphic Routing Network (ITS/SCPST)**. It binds the standalone cryptographic crate (`ITS`), hardware drivers (`ITS-hardware`), and distributed storage vaults (`ITS-ledger`) into an impenetrable, noise-immune communication network.

### 5-Tier Ecosystem Structure:
```
                    ITS High-Assurance Architecture Placement
                    ┌────────────────────────────────────────┐
                    │          ITS-net (This Repository)     │
                    │      CLI Daemon, Transport & I/O       │
                    └───────────────────┬────────────────────┘
                                        │
         ┌──────────────────────────────┼──────────────────────────────┐
         ▼                              ▼                              ▼
┌──────────────────┐          ┌──────────────────┐          ┌──────────────────┐
│    ITS-crypto    │          │   ITS-hardware   │          │    ITS-ledger    │
│  Core mathematical│          │ FFI isolation,   │          │ Local vaulting,  │
│ formulas & fields│          │ CRF & EM blinding│          │ registries & AEH │
└──────────────────┘          └──────────────────┘          └──────────────────┘
                                        │
                                        ▼
                         ┌──────────────────────────────┐
                         │  ITS-self_enclosed_timelock  │
                         │ Hybrid SSS-Chained Time-Lock │
                         └──────────────────────────────┘
                         ┌──────────────────────────────┐
                         │ ITS-OTM_public_attestation   │
                         │ Public Wegman-Carter OTM     │
                         └──────────────────────────────┘
```

**Upstream crates:**
* [`ITS-self_enclosed_timelock`](https://github.com/0x1F464/ITS-self_enclosed_timelock) — `its-net time-lock`, `time-unlock`, `time-deny`
* [`ITS-OTM_public_attestation`](https://github.com/0x1F464/ITS-OTM_public_attestation) — AEH/sneakernet OTM verify via `verify_public_otm_tag`; standalone `its_otm` CLI for public bundles

---

## 2. High-Assurance Documentation Portal

To satisfy strict academic peer-reviews and network-level security audits, the formal documentation suite of this repository is structured into six dedicated high-assurance documents in this directory:

1.  **[ITS-net_vision.md](ITS-net_vision.md) (Network-Level Threat Model & Transition Strategy)**
    *   Defines the network threat landscape: global traffic analysis, passive router correlation, active packet injection, and the tactical choice between active onion routing (Option A) and passive entropy harvesting (Option B).
2.  **[ITS-net_mathematics.md](ITS-net_mathematics.md) (Formally Proven Network & Traffic Obfuscation Proofs)**
    *   Rigorous mathematical proofs for:
        *   **Constant-Rate Chaffing + Lorenz Chaotic Jitter:** Proving that the cross-correlation function $R_{xy}(\tau)$ between any two nodes is zero for all non-trivial delays, rendering global timing analysis entirely blind.
        *   **Morphic Mixing Blind Linear Mixing:** Proof of information-theoretic blindness via Rank-Nullity of underdetermined matrices.
3.  **[ITS-net_manual.md](ITS-net_manual.md) (Command-Line Reference, Configurations & Operations Guide)**
    *   Complete CLI guide for `its-net` daemon operations, time-lock puzzles, steganographic sending, and configuration file syntax.
4.  **[ITS-net_troubleshooting.md](ITS-net_troubleshooting.md) (Anomaly Detection, Anonymity Drifts & Recovery Procedures)**
    *   Details active network anomaly detection algorithms, out-of-order counter synchronization, UDP packet drops, automatic node rerouting, and time-lock CLI recovery.
5.  **[ITS-net_usecase.md](ITS-net_usecase.md) (Transport Use-Cases & Integration Guide)**
    *   Tactical deployment scenarios including air-gapped time-lock custody via `ITS-self_enclosed_timelock`.
6.  **[ITS-net_HEADS_UP.md](ITS-net_HEADS_UP.md) (Tactical Threat Profile & Worst-Case Survival Guide)**
    *   Physical-layer constraints, duress protocols, and offline time-lock workflow.

---

## 3. Build & Verification Guide

### Compilation:
This crate compiles natively under Rust 2021 Edition.
```bash
cargo build --release
```

### Run the Integration and Network Simulation Tests:
```bash
cargo test
```
