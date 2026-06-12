# ITS / SCPST High-Assurance Specification & Verification Suite
## License: GNU GPLv3 Only
## Target: Academic & Peer-Review Audit

Welcome to the formal verification and audit documentation suite for the Information-Theoretic Secrecy (ITS) / Shamir-Chained Perfect Secrecy Trapdoor (SCPST) ecosystem. 

This repository and all associated sub-modules are released under the terms of the **GNU General Public License, Version 3 (GPLv3)**. See the `LICENSE` file in the root directory of this repository for full license details.

```
                    ITS High-Assurance Specification Framework
                    ┌──────────────────────────────────────┐
                    │            spec/README.md            │
                    │   Introduction & GPLv3 Licensing     │
                    └──────────────────┬───────────────────┘
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         ▼                             ▼                             ▼
┌──────────────────┐          ┌──────────────────┐          ┌──────────────────┐
│ spec/mathematics │          │  spec/systems_   │          │  spec/hardware_  │
│      .md         │          │   software.md    │          │  sidechannel.md  │
│  ITS proofs &    │          │ Hukommelsessikker│          │ CRF, AEH, Lorenz │
│ Field algebra    │          │ hed & seL4 pages │          │  & Jitter models │
└──────────────────┘          └──────────────────┘          └──────────────────┘
```

---

## 1. Scope of the Specification Suite

This directory is designed specifically for academic professors, peer-review teams, and independent cryptographic auditors to analyze and mathematically verify the security guarantees of our decoupled multi-repository architecture.

The specification suite is divided into three distinct technical domains:

1.  **[spec/mathematics.md](mathematics.md) (Domain: Mathematics & Cryptography)**
    *   Defines our modular finite fields ($\mathbb{Z}_{2^{31}-1}$ and $\mathbb{Z}_{2^{61}-1}$).
    *   Proves Information-Theoretic Secrecy (ITS) over Shamir's Secret Sharing (SSS) chained structures.
    *   Establishes the upper bound for Wegman-Carter One-Time Message (OTM) forgery probability.
    *   Proves the mathematical validity of Asymmetric Key-less Bootstrapping (Zero Prior Secret).

2.  **[spec/systems_software.md](systems_software.md) (Domain: Computer Science & Systems Security)**
    *   Defines the compiler barrier boundaries and memory sanitization via `Zeroize`.
    *   Analyzes constant-time execution paths to eliminate cache and branch side-channels.
    *   Specifies seL4 aligned process shared page parameters (`Sel4SharedPage` aligned to 4KB).

3.  **[spec/hardware_sidechannel.md](hardware_sidechannel.md) (Domain: Hardware, TEMPEST & Analog Interfaces)**
    *   Defines Cryptographic Reverse Firewalls (CRF) to neutralize subliminal kleptographic leakage on backdoored processors.
    *   Specifies Ambient Entropy Harvesting (AEH) immunity under hostile external source injection.
    *   Models TEMPEST electromagnetic and power slurring via Lorenz chaos-based timing jitter.
    *   Details the analog hex format with Adler-32 checksums for air-gapped physical share transfers.

---

## 2. Decoupled Crate Architecture

To preserve high auditability, the ecosystem is strictly partitioned into distinct private repositories:

*   **`0x1F464/ITS`:** Standalone, sterile, `no_std` krypto-kasse containing `core_logic` arithmetic.
*   **`0x1F464/ITS-net`:** OS transport layer containing the network dæmon (`its_net_cli`), UDP courier, and routing.
*   **`0x1F464/ITS-ledger`:** Blockchain syncer, public trapdoor registry, and distributed consensus mechanisms.
*   **`0x1F464/ITS-hardware`:** Hardware drivers, physical TRNG integrations, and CRF hardware gate-array software simulators.

---

## 3. How to Run the Verification Tests

### Unit and Algebraic Tests
To verify all mathematical operations, navigate to your clone of the core `ITS` cryptolayer repository and execute:
```bash
cargo test --all-features
```
This runs all 35 tests, including Barrett reduction validations, polynomial interpolations, and SSS chained time-locks.

### System Integration Tests
To run network simulations, decoy evaluations, and transport integrations, navigate to the `ROUTING` workspace root and execute:
```bash
cargo test
```
