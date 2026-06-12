# Hardware, Side-Channel & Analog Interface Specification
## License: GNU GPLv3 Only
## Target: Hardware Engineers, TEMPEST Auditors & Embedded Security Researchers

This document details the hardware-level threat mitigations implemented in the ITS/SCPST ecosystem, specifically addressing hardware trojans, TEMPEST side-channels, and physical air-gapped data transfers.

---

## 1. Cryptographic Reverse Firewalls (CRF)

Standard software-level isolation crashes if the underlying physical CPU or TRNG peripheral has been maliciously backdoored by a manufacturer or state actor (Trojan Horse). 

### Subliminal Kleptographic Leakage:
A compromised CPU can silently leak bits of the master keys by subtly modulating the least significant bits of public coordinates, public keys, or signatures (e.g., using a kleptographic setup where only the attacker can decode the leakage). This is a silent, un-detectable attack because the public output appears completely random and mathematically valid.

### Overwriting Leakage via CRF:
To neutralize this threat, our software architecture is designed to pass public points through a **Cryptographic Reverse Firewall (CRF)**. A CRF is a simple, physically clean, and formally verified hardware gate-array placed between the computing node and the public network.
*   **Morphic Overwrite:** The CRF intercepts Alice's public coordinates $(x_i, y_i)$ and multiplies them with a clean, locally generated random scalar $r \in \mathbb{F}_p$.
*   **Leakage Destruction:** This process maintains the underlying mathematical consistency required for Bob's transposition but completely destroys and overwrites any subliminal bit-modulation leakage. The trojan-horse CPU loses all freedom of output representation, neutralizing the hardware attack instantly.

---

## 2. Passive Entropy Parasitism (PEP) Immunity

If an adversary poisons the system's local TRNG (Entropy Poisoning), standard pseudo-random generators will output predictable key streams, destroying all cryptographic security.

### External Entropy Parasitism:
To defeat local TRNG poisoning, our system implements **Passive Entropy Parasitism (PEP)**. The daemon passively monitors external, high-velocity public data streams (such as live stock market tickers, weather sensors, or blockchain hash feeds).
*   **Universal Chaining Blend:** This external public data is blended with our local entropy pool using our **ITS-Secure Universal Polynomial Chaining** algorithm (rather than traditional hashes like SHA256).
*   **Mathematical Boundary:** The blending is designed such that even if Eve fully controls and poisons the external public source, she cannot bias the resulting pool unless she also possesses Alice and Bob's private trapdoor. The external source is treated as a pure, high-entropy noise source, and the mathematical boundaries guarantee that any malicious bias is immediately neutralized.

---

## 3. TEMPEST & Power Signature Jitter

Computers radiate electromagnetic waves and consume varying amounts of power depending on the CPU instructions being executed. An observer with an oscilloscope or radio receiver (TEMPEST attack) can reconstruct private keys by analyzing these physical signals.

### Lorenz Chaotic Timing Jitter:
To slur the physical signature of the CPU, we integrate chaotic timing delays during all sensitive cryptographic calculations.
*   **Lorenz Attractor Model:** The delay sequence is governed by a chaotic Lorenz Attractor calculated over the finite field:
    $$ x_{k+1} = x_k + \sigma (y_k - x_k) \cdot dt \pmod p $$
    $$ y_{k+1} = y_k + (x_k (\rho - z_k) - y_k) \cdot dt \pmod p $$
    $$ z_{k+1} = z_k + (x_k y_k - \beta z_k) \cdot dt \pmod p $$
*   **Computational Noise:** The resulting chaotic sequence is used to inject random computation loops and memory dummy-accesses (computational jitter). This slurs the processor's electromagnetic and power profile, making it impossible for TEMPEST equipment to distinguish cryptographic operations from background noise.

---

## 4. Analog & Air-Gapped SSS Share Transfers

When transferring keys or fragments over physical/analog mediums (such as handwritten paper, visual optical codes, or vocal read-outs) to maintain absolute air-gaps, we must prevent human translation errors.

### Checked Analog Share Format:
Shamir's Secret Sharing (SSS) shares are exported into an ASCII-standardized hexadecimal format with a robust trailing checksum.
*   **Adler-32 Checksum:** To protect against bit flips, typos, or character swaps, each printed or handwritten share includes an integrated Adler-32 checksum appended to the payload.
*   **Formatting Structure:** Shares are represented as formatted blocks of hex strings separated by hyphens (e.g., `DATA-DATA-CHECKSUM`).
*   **Verification:** During the CLI command `client-import-share` (implemented in `hydra_cli/src/main.rs`), the parser verifies the checksum before performing Lagrange interpolation, guaranteeing that no corrupt shares can propagate into the mathematical reconstruction layer.
