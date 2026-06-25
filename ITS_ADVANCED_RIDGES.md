# ITS Advanced Ridges — not Constitution essentials

## License: GNU GPLv3 Only

**Default operator path:** [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) — seven essentials only (`its-km`, `its-routing`, `its_asymmetric`, `routing.toml`, PATH, QR/manual keys).

This document describes **optional ridges** — subprocesses or Cargo features that extend transport or math. They are **not** required for constitution send/receive.

---

## Constitution vs Advanced

| Tier | Repos / binaries | When to use |
|------|------------------|-------------|
| **Constitution (7)** | ROUTING, ITS-KeyManagement, ITS-asymmetric | Default messaging, vault, pool carrier |
| **Full C2 (optional)** | ITS-OTM (`its_otm`) | Attestation tiers on wire — not in the 7 |
| **Ridges (advanced)** | timelock, FE, SSS_CHAIN, hardware, ledger | Feature-gated or air-gap specialist workflows |

---

## Ridge reference

| Ridge | Binary / feature | Role | Constitution path |
|-------|------------------|------|-------------------|
| **OTM** | `its_otm` / `--features otm` | WC-MAC attestation on AEH blocks | KM may pipe verify before send |
| **Timelock** | `its-km timelock` → `its-routing time-lock` | Coercion deniability (C4) | **Preferred:** KM orchestrates routing subcommand |
| **Standalone timelock** | `its_timelock` (timelock repo) | Puzzle generation without full routing build | Air-gap puzzle prep only |
| **Fingerprint erasure** | `its_fe` / `--features fingerprint-erasure` | Γ CR-NF before send | Optional on routing send |
| **SSS analog** | `client-export-share` / `client-import-share` | Physical share strings | Requires `--features hardware` |
| **Ledger** | `its_ledger` | Endpoint registry + AEH hash feed | **Not** operator vault — see below |
| **Hardware** | sidechannel_resistant_hardware / `--features hardware` | TRNG, analog export | Optional entropy |

---

## Timelock — single operator story

1. **Constitution (default):** `its-km timelock --contact ALIAS ...` → subprocess to `its-routing time-lock` / `time-unlock` with vault keys on PATH.
2. **Standalone:** `its_timelock generate` in the timelock repo — for air-gap puzzle files **without** building routing ridges.

Do not mix standalone puzzle paths with KM receive without reading [ITS-KeyManagement_manual.md](../ITS-KeyManagement/ITS-KeyManagement_manual.md).

---

## Ledger demotion

**ITS-KeyManagement** (`its-km`, `~/.its/km.vault`) is the operator keyring.

**ITS-ledger** (`its_ledger`) is endpoint-side registry + public AEH hash feed — advanced infrastructure, not the default vault story. See [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md#two-vaults-do-not-confuse).

---

## Links

- [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) — default operator path (seven essentials)  
- [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md) — layer rules and repo register  
- [ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md) — I2P/Nym feature map (constitution gates)  
- [ITS_PIPE_STDIO_POLICY.md](ITS_PIPE_STDIO_POLICY.md) — which CLIs support `-` stdin/stdout  
- [ITS-routing_PREBUILD_DOCTRINE.md](ITS-routing_PREBUILD_DOCTRINE.md) — build profiles and gates
