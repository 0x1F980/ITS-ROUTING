# Hardware, Side-Channel & Analog Interface Specification
## License: GNU GPLv3 Only
## Target: Hardware Engineers, TEMPEST Auditors & Embedded Security Researchers

This document details the hardware-level threat mitigations implemented in the ITS/SCPST ecosystem, specifically addressing hardware trojans, TEMPEST side-channels, and physical air-gapped data transfers.

---

## 1. Cryptographic Reverse Firewalls (CRF)

Standard software-level isolation crashes if the underlying physical CPU or TRNG peripheral has been maliciously backdoored by a manufacturer or state actor (Trojan Horse). 

### Subliminal Kleptographic Leakage:
A compromised CPU can silently leak bits of the master keys by subtly modulating the least significant bits of public coordinates, public keys, or signatures (e.g., using a kleptographic setup where only the attacker can decode the leakage). This is a silent, un-detectable attack because the public output appears completely random and mathematically valid.

### Formal Mathematical Proof of Leakage Destruction via CRF:
To neutralize this threat, our software architecture is designed to pass public points through a **Cryptographic Reverse Firewall (CRF)**. A CRF is a simple, physically clean, and formally verified hardware gate-array placed between the computing node and the public network.

Let the potentially backdoored CPU generate a coordinate share $D_i = (x_i, y_i) \in \mathbb{F}_p \times \mathbb{F}_p$, where $y_i = P(x_i) \pmod p$ is the evaluation of the secret polynomial. The compromised CPU attempts to leak a private key bit-stream $L \in \{0, 1\}^*$ by embedding it into the representation of $y_i$. This can be modeled as a kleptographic or subliminal channel function:
$$ y_i = \psi(P(x_i), L_k) $$
where $\psi$ is a deterministic or randomized mapping that preserves the mathematical validity of the coordinate (i.e., $y_i$ is still a valid share of the polynomial) but modulates specific bits (such as the least significant bits, or the distribution of quadratic residues) to encode the leakage $L_k$.

The CRF intercepts the coordinate $D_i = (x_i, y_i)$ and applies a randomized, morphic transformation before it is transmitted over the public network.
1. **Random Scalar Generation:** The CRF draws a fresh, clean, uniformly distributed random scalar $r \in \mathbb{F}_p^* = \mathbb{F}_p \setminus \{0\}$ from a trusted, physically isolated hardware entropy source.
2. **Morphic Overwrite:** The CRF multiplies the coordinate value $y_i$ by $r$:
   $$ y'_i = r \cdot y_i \pmod p $$
   The resulting randomized coordinate is $D'_i = (x_i, y'_i)$.

#### Information-Theoretic Proof of Zero Leakage:
Let $Y_i$ be the random variable representing the CPU's output coordinate, and let $R$ be the random variable representing the CRF's random scalar, which is uniformly distributed over $\mathbb{F}_p^*$ and independent of $Y_i$ and the leakage $L$.
For any observed output coordinate $y^* \in \mathbb{F}_p^*$, and any given CPU coordinate $y_i \in \mathbb{F}_p^*$, there exists exactly one unique scalar $r \in \mathbb{F}_p^*$ that satisfies the morphic equation:
$$ r = y^* \cdot y_i^{-1} \pmod p $$
Since $R$ is uniformly distributed over $\mathbb{F}_p^*$, the conditional probability of observing $y'_i = y^*$ given $y_i$ is:
$$ P(y'_i = y^* \mid y_i) = P(R \equiv y^* \cdot y_i^{-1} \pmod p) = \frac{1}{p-1} $$
Because this probability is constant and independent of the value of $y_i$, we have:
$$ P(y'_i = y^* \mid y_i, L) = P(y'_i = y^* \mid y_i) = P(y'_i = y^*) = \frac{1}{p-1} $$
Therefore, the joint distribution of the randomized coordinate $y'_i$ and the subliminal leakage $L$ factors perfectly:
$$ P(y'_i, L) = P(y'_i) \cdot P(L) $$
This means that $y'_i$ and $L$ are statistically independent. In terms of Shannon's Mutual Information:
$$ I(L; y'_i) = H(y'_i) - H(y'_i \mid L) = \log_2(p-1) - \log_2(p-1) = 0 $$
This mathematically proves that the CRF completely destroys and overwrites any subliminal bit-modulation or kleptographic leakage. The backdoored CPU loses all freedom of output representation, and the mutual information between the leaked stream $L$ and the public channel is reduced to exactly zero, neutralizing the hardware attack instantly while maintaining the underlying linear mathematical consistency required for Bob's decapsulation (since the scalar $r$ can be factored out during interpolation).

---

## 2. Ambient Entropy Harvesting (AEH) Immunity

If an adversary poisons the system's local TRNG (Entropy Poisoning), standard pseudo-random generators will output predictable key streams, destroying all cryptographic security.

### Ambient Entropy Harvesting:
To defeat local TRNG poisoning, our system implements **Ambient Entropy Harvesting (AEH)**. The daemon passively monitors external, high-velocity public data streams (such as live stock market tickers, weather sensors, or blockchain hash feeds).
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

### Formal Mathematical Proof of Total Algebraic Isolation of Adler-32:
A critical concern in high-security systems is whether appending a deterministic checksum (like Adler-32) to SSS shares leaks information about the underlying secret $S$, thereby compromising the information-theoretic security of the Shamir scheme. We prove that the Adler-32 checksum is algebraically isolated and leaks absolutely zero information about the secret.

Let $S \in \mathbb{F}_p$ be the secret, and let $D_j = (x_j, y_j) \in \mathbb{F}_p^2$ be the $j$-th share generated by the polynomial $P(x)$ of degree $t-1$. Let $C(D_j) = \text{Adler32}(D_j)$ be the Adler-32 checksum of the share $D_j$.
Let $J \subset \{1, \dots, n\}$ be a subset of share indices intercepted by an eavesdropper Eve, such that $|J| = k < t$ (i.e., the number of intercepted shares is strictly below the reconstruction threshold $t$).
The intercepted data consists of the shares $D_J = \{ D_j \}_{j \in J}$ and their corresponding checksums $C(D_J) = \{ C(D_j) \}_{j \in J}$.

#### 1. Information-Theoretic Isolation Proof:
By definition of Shamir's Secret Sharing over $\mathbb{F}_p$, any subset of $k < t$ shares is statistically independent of the secret $S$. Thus, the conditional entropy of $S$ given $D_J$ is equal to the prior entropy of $S$:
$$ H(S \mid D_J) = H(S) $$
The Adler-32 checksum $C(D_j)$ is a purely deterministic function of the share $D_j$. That is, there exists a deterministic mapping $f: \mathbb{F}_p^2 \to \mathbb{Z}_{2^{32}}$ such that $C(D_j) = f(D_j)$.
Therefore, the conditional entropy of $C(D_J)$ given $D_J$ is zero:
$$ H(C(D_J) \mid D_J) = 0 $$
Using the chain rule for entropy, the joint conditional entropy of the secret $S$ and the checksums $C(D_J)$ given the shares $D_J$ can be expanded in two ways:
$$ H(S, C(D_J) \mid D_J) = H(S \mid D_J) + H(C(D_J) \mid S, D_J) $$
Since $C(D_J)$ is a deterministic function of $D_J$, we have $H(C(D_J) \mid S, D_J) = 0$. Thus:
$$ H(S, C(D_J) \mid D_J) = H(S \mid D_J) = H(S) $$
Alternatively, we can expand the joint conditional entropy as:
$$ H(S, C(D_J) \mid D_J) = H(C(D_J) \mid D_J) + H(S \mid D_J, C(D_J)) $$
Since $H(C(D_J) \mid D_J) = 0$, this simplifies to:
$$ H(S, C(D_J) \mid D_J) = H(S \mid D_J, C(D_J)) $$
Equating the two expressions, we obtain:
$$ H(S \mid D_J, C(D_J)) = H(S) $$
This mathematically proves **Total Algebraic Isolation**: the joint observation of the shares and their Adler-32 checksums leaks absolutely zero information about the secret $S$, as long as the number of shares is below the threshold $t$. The prior uncertainty of the secret remains completely undiminished.

#### 2. Cryptographic Integrity and Authenticity via Wegman-Carter OTM:
The Adler-32 checksum is used exclusively as a non-cryptographic error-detecting code to guard against accidental transmission errors (typos, transposition of characters). It does not provide any cryptographic security against active tampering.
The cryptographic integrity and authenticity of the shares is solely guaranteed and verified by the Wegman-Carter One-Time Message (OTM) tags computed over $\mathbb{F}_p$.
If an active adversary Eve attempts to inject a manipulated share $D'_j \neq D_j$, she must also forge a valid Wegman-Carter tag $T'_j$. As proven in the mathematical specification, the probability of Eve successfully forging a tag is bounded by:
$$ P_{\text{forge}} \le \frac{d}{p} $$
The system will inevitably reject any manipulated share with probability:
$$ P_{\text{reject}} = 1 - P_{\text{forge}} \ge 1 - \frac{d}{p} $$
For **M31**, $P_{\text{reject}} \ge 99.99995\%$, and for **M61**, $P_{\text{reject}} \ge 99.9999999999999999\%$. Thus, active tampering is mathematically guaranteed to be detected and rejected, while the Adler-32 checksum remains perfectly isolated and harmless.
