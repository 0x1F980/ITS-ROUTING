# Algebraic & Cryptographic Specification (ITS/SCPST)
## License: GNU GPLv3 Only
## Target: Mathematicians, Cryptographers & Peer-Reviewers

This document provides the formal mathematical and algebraic specification of the SSS-Chained Perfect Secrecy Trapdoor (SCPST) protocol. All calculations and proofs are designed under the assumption of an **omnipotent Eve** controlling all transit channels and hardware, except the local abstract intentions of Alice and Bob.

---

## 1. Prime Field Parameterization (\mathbb{Z}_p)

The core cryptographic operations are performed over a prime field $\mathbb{F}_p = \mathbb{Z}_p$. The system is configurable via Cargo features to support two primary Mersenne prime moduli:

### Prime Fields:
1.  **M31 Field (Default):**
    $$ p = 2^{31} - 1 = 2147483647 $$
    This is a Mersenne prime ($M_{31}$). Reduction can be performed extremely rapidly using bitwise shifts.
2.  **M61 Field (m61 feature):**
    $$ p = 2^{61} - 1 = 2305843009213693951 $$
    This is a Mersenne prime ($M_{61}$), providing an exponentially larger security margin and lower collision probability.

### Constant-Time Barrett Reduction:
To avoid costly division instructions (which are non-constant time on most CPUs), modular reduction is implemented using a constant-time variant of Barrett reduction.
For any double-width integer $z \in [0, p^2]$, we compute:
$$ q = \lfloor \frac{z \cdot \mu}{2^k} \rfloor $$
$$ r = z - q \cdot p $$
$$ \text{if } r \ge p \text{ then } r = r - p $$
Where $\mu = \lfloor \frac{2^k}{p} \rfloor$. Under the prime configuration, this is represented using conditional selection (`subtle::Choice`) to ensure complete protection against timing side-channels.

---

## 2. Shamir's Secret Sharing (SSS) Over \mathbb{Z}_p

Data is fragmented into $n$ shares such that any threshold $t$ shares can reconstruct the master secret $S$, while any $t-1$ shares yield **zero information** about $S$ (Information-Theoretic Secrecy).

### Share Generation:
A polynomial $P(x)$ of degree $t-1$ is constructed over $\mathbb{F}_p$:
$$ P(x) = S + a_1 x + a_2 x^2 + \dots + a_{t-1} x^{t-1} \pmod p $$
Where $S \in \mathbb{F}_p$ is the secret payload byte or token, and $a_i \in \mathbb{F}_p$ are independent, uniformly distributed random coefficients drawn from our `SecureRandom` source.
Shares are represented as coordinates:
$$ D_i = (x_i, P(x_i)) \pmod p \quad \text{for } x_i \in [1, n] $$

### Lagrange Interpolation:
Given $t$ distinct shares $(x_i, y_i)$, the secret $S = P(0)$ is reconstructed using Lagrange basis polynomials evaluated at $x = 0$:
$$ S = \sum_{i=1}^t y_i \cdot \ell_i(0) \pmod p $$
$$ \ell_i(0) = \prod_{j \neq i} \frac{-x_j}{x_i - x_j} \pmod p $$
All modular divisions in interpolation are solved using Fermat's Little Theorem:
$$ a^{-1} \equiv a^{p-2} \pmod p $$
Computed via constant-time modular exponentiation (square-and-multiply).

---

## 3. SSS-Chained Time-Lock Puzzles

Our deniable time-lock puzzle combines an RSW96 sequential squaring puzzle (computational delay) with a 1-to-1 SSS chain (information-theoretic delay).

### RSW96 Component:
An RSA-modulus $N = q_1 \cdot q_2$ is defined. For a given difficulty $T$ (squaring steps), we compute:
$$ y = x^{2^T} \pmod N $$
This computation is intrinsically sequential, preventing parallel acceleration by an adversary.

### SSS-Chain Component:
For each squaring step $i \in [1, T]$, we define a 1-to-1 Shamir sharing $P_i(x)$ over $\mathbb{F}_p$ such that:
$$ P_i(0) = K_i \quad (\text{The key/share of step } i) $$
The starting share $P_1(1)$ is given publicly. Decrypting step $i$ mathematically requires recovering $P_i(0)$, which serves as the key to unlock the SSS polynomial for step $i+1$.

### Mathematical Proof of Perfect Deniability:
Because SSS is information-theoretically secure, any arbitrary guess for an uncomputed share $P_k(x)$ will interpolate to a mathematically consistent, valid secret $S'$:
$$ \forall S' \in \mathbb{F}_p, \exists (a_1, \dots, a_{t-1}) \text{ s.t. } P(x_i) = y_i $$
Therefore, an attacker with infinite computing power cannot distinguish whether a recovered message is the "true" message or a plausible decoy, guaranteeing complete deniability under duress.

---

## 4. Wegman-Carter One-Time Message (OTM) Tags

To secure message integrity without relying on SHA256 or other computationally-reducible hashes, we implement ITS-secure Wegman-Carter MAC tags via **ITS-Secure Universal Polynomial Chaining**.

### Polynomial Evaluation:
Let the message be parsed as a sequence of field elements $m_1, m_2, \dots, m_k \in \mathbb{F}_p$. We define a polynomial:
$$ H_x(m) = m_1 x^k + m_2 x^{k-1} + \dots + m_k x \pmod p $$
The key $x \in \mathbb{F}_p$ is a shared, one-time secret. The final integrity tag $T$ is computed by adding an independent masking secret $s \in \mathbb{F}_p$:
$$ T = H_x(m) + s \pmod p $$

### Mathematical Proof of Forgery Bound:
The polynomial family is $\Delta$-universal. If an attacker Eve intercepts a valid pair $(m, T)$ and attempts to forge $(m', T')$, the probability of a successful forgery $P_{\text{forge}}$ is bounded by:
$$ P_{\text{forge}} \le \frac{d}{p} $$
Where $d = \max(k)$ is the degree of the polynomial (message length in field elements), and $p$ is the modulus.
* For **M31**, even with a message length of 1024 elements, $P_{\text{forge}} < 10^{-6}$.
* For **M61**, $P_{\text{forge}} < 10^{-15}$, which is mathematically equivalent to absolute physical impossibility.

---

## 5. Asymmetric ITS Key-less Bootstrapping

We resolve the bootstrapping paradox (requiring an out-of-band pre-shared key) by using Bob's public trapdoor for information-theoretically secure encapsulation.

### Protocol Steps:
1.  **Bob's Private Trapdoor:** Bob defines a private evaluation coordinate $x_{\text{Bob}} \in \mathbb{F}_p$ on a high-degree polynomial $P(x)$ over $\mathbb{F}_p$. He keeps $x_{\text{Bob}}$ completely private.
2.  **Bob's Public Points:** Bob publishes a subset of $d$ points:
    $$ D_{\text{pub}} = \{ (x_i, P(x_i)) \} \quad \text{for } i \in [1, d], \text{ where } x_i \neq x_{\text{Bob}} $$
3.  **Alice's Encapsulation:** Alice selects a secret $K_{\text{pool}}$ and a random masking polynomial $Q(x)$ of degree $d-1$. She encapsulates $K_{\text{pool}}$ by adding it to Bob's public points, producing:
    $$ y'_i = P(x_i) + Q(x_i) \cdot K_{\text{pool}} \pmod p $$
4.  **Decapsulation:** Because Lagrange interpolation is linear, Bob can evaluate the composite polynomial at his private coordinate $x_{\text{Bob}}$:
    $$ P(x_{\text{Bob}}) + Q(x_{\text{Bob}}) \cdot K_{\text{pool}} \pmod p $$
    Since Bob knows the private relation, he extracts $K_{\text{pool}}$ instantly.
5.  **ITS Security:** To Eve, the values $(y'_1, \dots, y'_d)$ represent an under-determined system of linear equations. There are infinitely many combinations of $Q(x)$ and $K_{\text{pool}}$ that yield the identical public points, rendering it mathematically impossible for Eve to recover $K_{\text{pool}}$ even with infinite computing power.
