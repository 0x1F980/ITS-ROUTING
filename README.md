# ITS-routing: OS Transport Layer & CLI Daemon (Transport/CLI)

## GNU General Public License v3.0 Only
Copyright (C) 2026 0x1F980. All rights reserved.

ITS-routing is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

**Operator identity:** [ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement) — contacts, vault, duress, orchestration. **Transport boundary:** [ITS-routing_KEEP_BOUNDARY.md](ITS-routing_KEEP_BOUNDARY.md).

---

## 1. Overview & Architecture

**Read first:** **[ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md)** — maps subcommands to upstream ITS scope docs. Ecosystem master: [ITS_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-ROUTING/blob/master/ITS_ECOSYSTEM.md).

`ITS-routing` (binary `its-routing`, crate `its_routing`) is the **transport engine** of the Morphic Routing Network (ITS/SCPST). It moves bytes on the wire — onion, UDP, SSS, chaff. It does **not** manage human contacts or vault passwords (see ITS-KeyManagement).

### Ecosystem placement:
```
                    ┌────────────────────────────────────────┐
                    │     ITS-KeyManagement (identity + orchestration)  │
                    │  vault, contacts, its-km send/receive  │
                    └───────────────────┬────────────────────┘
                                        │ subprocess pipes
                    ┌───────────────────▼────────────────────┐
                    │     ITS-routing (This Repository)      │
                    │   CLI daemon, transport & expert pipes   │
                    └───────────────────┬────────────────────┘
         ┌──────────────────────────────┼──────────────────────────────┐
         ▼                              ▼                              ▼
┌──────────────────┐          ┌──────────────────┐          ┌──────────────────┐
│  ITS-asymmetric  │          │   ITS-hardware   │          │    ITS-ledger    │
│  wire v6 static  │          │ FFI isolation    │          │ node registries  │
└──────────────────┘          └──────────────────┘          └──────────────────┘
         │                              │
         ▼                              ▼
  ITS-OTM / timelock / fingerprint-erasure (expert pipes also via its-routing CLI)
```

**Upstream crates:**
* [`ITS-self_enclosed_timelock`](https://github.com/0x1F980/ITS-self_enclosed_timelock) — `its-routing time-lock`, `time-unlock`, `time-deny`
* [`ITS-OTM_public_attestation`](https://github.com/0x1F980/ITS-OTM_public_attestation) — AEH/sneakernet OTM verify via `verify_public_otm_tag`; standalone `its_otm` CLI for public bundles
* [`ITS-FINGERPRINT_ERASURE`](https://github.com/0x1F980/ITS-FINGERPRINT_ERASURE) — Γ extended mode + two-domain NF; `--fingerprint-erasure` enforces strict stack (OTP+chaff); `its_fe watch` for post-save; `--fe-permissive` escape

---

## 2. High-Assurance Documentation Portal

To satisfy strict academic peer-reviews and network-level security audits, the formal documentation suite of this repository is structured into dedicated high-assurance documents in this directory:

0.  **[ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md) (Security Layers & ITS Scope — read first)**
    *   Subcommand → upstream crate scope map; default Γ + OTP + chaff stack; anti-AI FAQ.
0b. **[ITS-routing_FORMAL_VERIFICATION.md](ITS-routing_FORMAL_VERIFICATION.md) (Formal Verification & W6 Composition)**
    *   Lean/Rust proof map for strict-stack send, OTM, timelock, fingerprint erasure; links to upstream ITS certificates.
0c. **[ITS-routing_PIPE.md](ITS-routing_PIPE.md) (stdin/stdout piping)**
    *   `-` paths for `time-lock`, `time-unlock`, `fingerprint-erasure`; demo `scripts/pipe_timelock.sh`.
1.  **[ITS-routing_vision.md](ITS-routing_vision.md) (Network-Level Threat Model & Transition Strategy)**
    *   Defines the network threat landscape: global traffic analysis, passive router correlation, active packet injection, and the tactical choice between active onion routing (Option A) and passive entropy harvesting (Option B).
2.  **[ITS-routing_mathematics.md](ITS-routing_mathematics.md) (Formally Proven Network & Traffic Obfuscation Proofs)**
    *   Rigorous mathematical proofs for:
        *   **Constant-Rate Chaffing + Lorenz Chaotic Jitter:** Proving that the cross-correlation function $R_{xy}(\tau)$ between any two nodes is zero for all non-trivial delays, rendering global timing analysis entirely blind.
        *   **Morphic Mixing Blind Linear Mixing:** Proof of information-theoretic blindness via Rank-Nullity of underdetermined matrices.
3.  **[ITS-routing_manual.md](ITS-routing_manual.md) (Command-Line Reference, Configurations & Operations Guide)**
    *   Complete CLI guide for `its-routing` daemon operations, time-lock puzzles, steganographic sending, and configuration file syntax.
4.  **[ITS-routing_troubleshooting.md](ITS-routing_troubleshooting.md) (Transport Recovery & Operational Procedures)**
    *   UDP packet loss via SSS redundancy, configuration drift, time-lock CLI recovery, and ratchet resync with ITS-KeyManagement.
5.  **[ITS-routing_usecase.md](ITS-routing_usecase.md) (Transport Use-Cases & Integration Guide)**
    *   Tactical deployment scenarios including air-gapped time-lock custody via `ITS-self_enclosed_timelock`.
6.  **[ITS-routing_HEADS_UP.md](ITS-routing_HEADS_UP.md) (Tactical Threat Profile & Worst-Case Survival Guide)**
    *   Physical-layer constraints, offline time-lock workflow; operator duress via **[ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement)**.
7.  **[ITS-routing_KEEP_BOUNDARY.md](ITS-routing_KEEP_BOUNDARY.md) (Transport vs operator identity boundary)**
    *   What stays in routing after the ITS-KeyManagement migration.

---

## 3. Build & Verification Guide

### Compilation:
This crate compiles natively under Rust 2021 Edition.
```bash
cargo build --release
```

### Run unit tests (`its_routing/`):
```bash
cd its_routing && cargo test
```
Tests cover analog SSS export/import roundtrip and stdin/stdout path detection. Integration tests for timelock, OTM verify, and fingerprint-erasure live in upstream crates (see [ITS-routing_FORMAL_VERIFICATION.md](ITS-routing_FORMAL_VERIFICATION.md)).
