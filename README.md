# ITS-routing: OS Transport Layer & CLI Daemon (Transport/CLI)

## GNU General Public License v3.0 Only
Copyright (C) 2026 0x1F980. All rights reserved.

ITS-routing is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

**ITS = Information-Theoretic Secrecy** — see [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md#its--information-theoretic-secrecy). Repo names (`ITS-*`) label components; they do not redefine the acronym.

**Operator identity:** [ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement) — contacts, vault, duress, orchestration. **Transport boundary:** [ITS-routing_KEEP_BOUNDARY.md](ITS-routing_KEEP_BOUNDARY.md).

---

## 1. Overview & Architecture

**Read first:** **[ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md)** — maps subcommands to upstream ITS scope docs. Ecosystem master: [ITS_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-ROUTING/blob/master/ITS_ECOSYSTEM.md).

`ITS-routing` (binary `its-routing`, crate `its_routing`) is the **transport engine** of the Morphic Routing Network (ITS/SCPST). **Production path:** UES Monocell Pool + CoverTransport + optional `its-pool-proxy`. It does **not** manage human contacts or vault passwords (see ITS-KeyManagement).

> **Dev-only:** onion/UDP/mix paths require `transport_mode = "dev"` or the `dev-onion-mix` feature — not the default prod narrative.

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
│  ITS-asymmetric  │          │ sidechannel_resistant_hardware │          │    ITS-ledger    │
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
0a. **[ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) (Mathematical Core — read for formal spec)**
    *   Authoritative axioms, C.I.A. formulas, Sybil/BIS/hops/timelock, Lean module map, v5 closure; prod = 0 hops UES Pool.
0b. **[ITS-routing_FORMAL_VERIFICATION.md](ITS-routing_FORMAL_VERIFICATION.md) (Formal Verification & W6 Composition)**
    *   Lean/Rust proof map for strict-stack send, OTM, timelock, fingerprint erasure; links to upstream ITS certificates.
0c. **[ITS-routing_PIPE.md](ITS-routing_PIPE.md) (stdin/stdout piping)**
    *   `-` paths for `time-lock`, `time-unlock`, `fingerprint-erasure`; demo `scripts/pipe_timelock.sh`.
1.  **[ITS-routing_vision.md](ITS-routing_vision.md) (Network-Level Threat Model & Transition Strategy)**
    *   Threat landscape under active Eve; **prod default = UES Pool (Option B)**. Option A onion routing is **dev-only** (`dev-onion-mix`).
2.  **[ITS-routing_mathematics.md](ITS-routing_mathematics.md) (Dev-only onion/mix proofs — historical)**
    *   Morphic mixing rank-nullity and Lorenz jitter — **dev-only** (`dev-onion-mix`). Production math: [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md).
3.  **[ITS-routing_manual.md](ITS-routing_manual.md) (Command-Line Reference, Configurations & Operations Guide)**
    *   Complete CLI guide for `its-routing` daemon operations, time-lock puzzles, steganographic sending, and configuration file syntax.
4.  **[ITS-routing_troubleshooting.md](ITS-routing_troubleshooting.md) (Transport Recovery & Operational Procedures)**
    *   Pool mirror failover, fountain recovery, time-lock CLI recovery, and ratchet resync with ITS-KeyManagement. (UDP/onion: dev-only.)
5.  **[ITS-routing_usecase.md](ITS-routing_usecase.md) (Transport Use-Cases & Integration Guide)**
    *   Tactical deployment scenarios including air-gapped time-lock custody via `ITS-self_enclosed_timelock`.
6.  **[ITS-routing_HEADS_UP.md](ITS-routing_HEADS_UP.md) (Tactical Threat Profile & Worst-Case Survival Guide)**
    *   Physical-layer constraints, offline time-lock workflow; operator duress via **[ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement)**.
7.  **[ITS-routing_KEEP_BOUNDARY.md](ITS-routing_KEEP_BOUNDARY.md) (Transport vs operator identity boundary)**
    *   What stays in routing after the ITS-KeyManagement migration.

---

## 3. Build & Verification Guide

**Primary transport (v2.0):** UES Monocell Pool — see [QUICKSTART.md](QUICKSTART.md) and [ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md).

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key send --contact bob --file doc.pdf
its-km --true-secret ~/.its/km-vault-keys/true/secret.key receive --contact alice --out received.pdf
```

UDP/onion paths remain for **dev** (`transport_mode = "dev"` in config). Production: copy `config.prod.toml` → `~/.its/routing.toml`.

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

### Shell completions

| Shell | Install |
|-------|---------|
| Bash | `source completions/its-routing.bash` |
| Zsh | `source completions/its-routing.zsh` |
| Fish | `cp completions/its-routing.fish ~/.config/fish/completions/` |
| PowerShell | `. ./completions/its-routing.ps1` |

### Build profiles

```bash
cargo build -p its_routing                              # transport only (default)
cargo build -p its_routing --features full              # all operational ridges
cargo build -p its_routing --features timelock,ledger    # custom subset
```

See [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md) for composable infrastructure philosophy and feature matrix.
