# ITS-net: Formally Proven Network & Traffic Obfuscation Proofs (ITS-net_mathematics)

## License: GNU GPLv3 Only
## Target: Mathematicians, Cryptographers & Traffic Analysis Auditors

> **Scope:** [ITS-net_SECURITY_LAYERS.md](ITS-net_SECURITY_LAYERS.md) — network proofs orchestrate upstream ITS layers.


This document provides formal mathematical and information-theoretic proofs for the traffic obfuscation and packet-mixing mechanisms implemented in `ITS-net`.

---

## 1. Mathematical Proof of Defense Against Timing-Correlation Attacks

Under standard onion routing, an active or passive global adversary (Eve) can perform timing correlation by matching packet arrival times at different nodes. If the packet rate is $R(t)$, Eve can compute the cross-correlation function $R_{xy}(\tau)$ between the incoming traffic $X(t)$ at the entry node and the outgoing traffic $Y(t)$ at the exit node:
$$R_{xy}(\tau) = \lim_{T \to \infty} \frac{1}{T} \int_0^T X(t) Y(t + \tau) \, dt$$
If $R_{xy}(\tau)$ exhibits a non-zero peak at some delay $\tau$, Eve can correlate the sender and receiver with high statistical confidence.

To completely prevent this, Morphic Routing Network (ITS/SCPST) combines **Constant-Rate Chaffing** with **Lorenz Chaotic Timing Jitter**:

1. **Constant-Rate Chaffing:** By maintaining a continuous stream of dummy packets (chaff) when no real payload is being transmitted, the overall packet rate $R(t) = R_{\text{const}}$ is kept as a flat, constant vector.
2. **Lorenz Chaotic Timing Jitter:** To prevent Eve from filtering out dummy packets based on periodic timing analysis (de-jittering), the packet transmission intervals are randomized using a Lorenz chaotic system. The Lorenz system is defined by three non-linear differential equations:
$$\frac{dx}{dt} = \sigma(y - x), \quad \frac{dy}{dt} = x(\rho - z) - y, \quad \frac{dz}{dt} = xy - \beta z$$
The chaotic trajectory is extremely sensitive to initial conditions (the butterfly effect). Any two trajectories starting with an infinitesimally small difference $\delta$ diverge exponentially over time:
$$| \Delta(t) | \approx e^{\lambda t} | \delta |$$
where $\lambda > 0$ is the positive Lyapunov exponent.

Because the chaotic intervals are non-periodic and deterministic but statistically indistinguishable from white noise without knowing the exact initial parameters (which are derived from the private key/ratchet), the output stream appears to Eve as a completely flat, invariant white noise vector:
$$S(f) = \text{constant}$$
This mathematically guarantees that the cross-correlation $R_{xy}(\tau)$ between any two nodes is zero for all non-trivial delays $\tau$:
$$R_{xy}(\tau) = 0 \quad \forall \tau$$
Thus, Eve is mathematically incapable of performing statistical timing correlation or de-jittering, rendering her global surveillance completely blind.

---

## 2. Mathematical Proof of Morphic Blindness (Packet Mixing)

During Morphic Routing, intermediate mixing nodes perform blind linear combinations of packets to defeat traffic analysis.

### The Underdetermined Mixing Proof:
Let $P_1, P_2 \in \mathbb{F}_p^L$ be two incoming masked packets, and let $C \in \mathbb{F}_p^L$ be the blended output:
$$ C = c_1 P_1 + c_2 P_2 \pmod p $$
where $c_1, c_2$ are public coefficients. Each packet $P_i$ is masked with an independent, uniform one-time key $K_i \in \mathbb{F}_p^L$:
$$ P_i = M_i + K_i \pmod p $$
The blended output is:
$$ C = c_1(M_1 + K_1) + c_2(M_2 + K_2) = (c_1 M_1 + c_2 M_2) + (c_1 K_1 + c_2 K_2) \pmod p $$

Let the state vector be $X = (M_1, M_2, K_1, K_2)^T \in \mathbb{F}_p^{4L}$. The observer Eve knows $C$ and the matrix of mixing coefficients:
$$ \mathbf{A} X = C \pmod p $$
where $\mathbf{A} \in \mathbb{F}_p^{L \times 4L}$ is a matrix of rank $\text{Rank}(\mathbf{A}) = L < 4L$.
By the Rank-Nullity Theorem, the dimension of the null space (kernel) of $\mathbf{A}$ is:
$$ \dim(\ker(\mathbf{A})) = 4L - L = 3L $$

Since the kernel has dimension $3L$, the solution set is an affine subspace of dimension $3L$ over $\mathbb{F}_p$. There are exactly $p^{3L}$ equally likely combinations of messages and keys that satisfy the observed blended output.
Therefore, Eve learns exactly $0$ bits of information about the individual messages $M_1$ and $M_2$, proving perfect morphic blindness.

---

## 3. Concrete Numerical Verification Walkthroughs

To prevent superficial analysis, we provide exact, step-by-step calculated values over the default Mersenne prime modulus $p = 2147483647$ ($2^{31}-1$).

### 1. Morphic Mixing Underdetermined Verification
Suppose a mixing node blends two incoming masked packets $P_1, P_2$ of size $L = 1$ with public coefficients $c_1 = 3$ and $c_2 = 5$.
The intercepted blended output value is $C = 10000$. Eve faces the equation:
$$ 3 \cdot (M_1 + K_1) + 5 \cdot (M_2 + K_2) \equiv 10000 \pmod p $$
$$ 3 \cdot M_1 + 5 \cdot M_2 + 3 \cdot K_1 + 5 \cdot K_2 \equiv 10000 \pmod p $$

Let us demonstrate how different candidate messages $(M'_1, M'_2)$ are supported by perfectly consistent, valid keys $(K'_1, K'_2)$ under the modulus:

* **Candidate 1: $M'_1 = 100$, $M'_2 = 200$**
  $$ 3 \cdot (100) + 5 \cdot (200) + 3 \cdot K_1 + 5 \cdot K_2 \equiv 10000 \pmod p $$
  $$ 300 + 1000 + 3 \cdot K_1 + 5 \cdot K_2 \equiv 10000 \implies 3 \cdot K_1 + 5 \cdot K_2 \equiv 8700 \pmod p $$
  If Bob's private trapdoor selects $K'_1 = 900$, we get:
  $$ 3 \cdot (900) + 5 \cdot K_2 \equiv 8700 \implies 2700 + 5 \cdot K_2 \equiv 8700 \implies 5 \cdot K_2 \equiv 6000 \pmod p $$
  $$ K'_2 = 1200 $$
  The keys $(K'_1=900, K'_2=1200)$ are perfectly consistent and uniform.

* **Candidate 2: $M'_1 = 500$, $M'_2 = 1000$**
  $$ 3 \cdot (500) + 5 \cdot (1000) + 3 \cdot K_1 + 5 \cdot K_2 \equiv 10000 \pmod p $$
  $$ 1500 + 5000 + 3 \cdot K_1 + 5 \cdot K_2 \equiv 10000 \implies 3 \cdot K_1 + 5 \cdot K_2 \equiv 3500 \pmod p $$
  If Bob's private trapdoor selects $K'_1 = 500$, we get:
  $$ 3 \cdot (500) + 5 \cdot K_2 \equiv 3500 \implies 1500 + 5 \cdot K_2 \equiv 3500 \implies 5 \cdot K_2 \equiv 2000 \pmod p $$
  $$ K'_2 = 400 $$
  The keys $(K'_1=500, K'_2=400)$ are perfectly consistent and uniform.

Since both candidates (and all other $p^3$ combinations) are mathematically identical, the mutual information $I(M_1, M_2; C) = 0$ bits, proving absolute morphic blindness.

### 2. Constant-Rate Chaffing Timing Verification
Suppose our target chaff transmission rate is configured to $R = 10$ packets per second (pps) over a window of $T = 1000$ ms.
The Lorenz Jitter generates non-periodic transmission ticks at intervals:
$$ I = [95, 105, 102, 98, 100, 101, 99, 103, 97, 100] \text{ ms} \quad (\sum I = 1000 \text{ ms}) $$

The daemon executes the send schedule at each discrete tick:
1. **Tick 1 (95ms):** Real packet $P_1$ exists in queue $\implies$ Send $P_1$.
2. **Tick 2 (200ms):** Queue is empty $\implies$ Generate dummy chaff packet $C_2$ and send.
3. **Tick 3 (302ms):** Queue is empty $\implies$ Generate dummy chaff packet $C_3$ and send.
4. **Tick 4 (400ms):** Real packet $P_4$ exists in queue $\implies$ Send $P_4$.
5. **Tick 5 (500ms):** Queue is empty $\implies$ Generate dummy chaff packet $C_5$ and send.
6. **Tick 6 (601ms):** Queue is empty $\implies$ Generate dummy chaff packet $C_6$ and send.
7. **Tick 7 (700ms):** Queue is empty $\implies$ Generate dummy chaff packet $C_7$ and send.
8. **Tick 8 (803ms):** Real packet $P_8$ exists in queue $\implies$ Send $P_8$.
9. **Tick 9 (900ms):** Queue is empty $\implies$ Generate dummy chaff packet $C_9$ and send.
10. **Tick 10 (1000ms):** Queue is empty $\implies$ Generate dummy chaff packet $C_{10}$ and send.

**Observed Profile:**
Eve records exactly 10 packets transiting the link during the 1.0 second window. The packet timing intervals are a non-correlated chaotic series $[95, 105, 102, 98, \dots]$, meaning Eve's cross-correlation function $R_{xy}(\tau) = 0$ for all non-trivial delays. Timing correlation is completely defeated.

---

## 4. Optional Γ (fingerprint erasure) — Eve channel matrix (v4 / v5)

When `--fingerprint-erasure` is enabled on send, payloads pass through Church-Rosser normalization before morphic mixing. Default is **permissive** (v3 sniff); opt-in **`--fe-strict`** requires explicit kind and denies Raw fallback; **`--fe-strict-stack`** is strict policy + extended quantization (no Raw).

**v5 two domains** (kinds are wire/implementation detail):

| Domain | Formula | `--fe-kind` values |
|--------|---------|-------------------|
| **Discrete** | $\Gamma_d = \mathrm{serialize}(\alpha(\mathrm{parse}(\mathrm{nfc}(B))))$ | `text`, `code`, `pdf` |
| **Continuous** | $\Gamma_c = \mathrm{template}^{-1}(\lfloor \mathcal{F}(\mathrm{decode}(B)) \rfloor_\Delta)$ | `image`, `audio` |

Optional **`--fe-domain discrete|continuous`** validates declared domain matches kind. Image $\mathcal{F}$ = block-DCT pipeline; audio = time-domain 44100 Hz stereo template (not spectral).

| Channel | v3 | v4 permissive | v4 strict | v5 strict stack + domain |
|---------|-----|---------------|-----------|-------------------------|
| Code identifiers | Yes | Yes | **No** (alpha-rename) | **No** |
| Polyglot routing | Yes | Yes | **No** (explicit kind) | **No** (+ cross-domain reject) |
| Audio encoding | Likely yes | Better resampler | **No** (44100 stereo) | **No** |
| Raw unknown | Yes | Yes | **No** (opt-in deny) | **No** |
| Confluence gap | Unknown | Fuzz tests | **Closed subset** (Lean) | Per-domain suites |
| Semantic sabotage | Yes | Yes | **Yes** (requires signature) | **Yes** |
| Hardware/timing | Yes | Yes | **Yes** (OTP/chaff/air-gap) | **Yes** (documented stack) |

Formal spec: sibling crate `ITS-fingerprint_erasure/mathematics/cr/` (`Cr/Discrete.lean`, `Cr/Continuous.lean`). Full formula table: `ITS-fingerprint_erasure_FORMULAS.md`. Wire OTP layer: $I(X; F) = 0$ after $\Gamma(M)$; full send-stack protection requires OTP + chaff + air-gap.

**v0.8 master stack:** $\mathcal{U}(M)=\text{Chaff}(\text{OTP}(\Gamma(M)))$. With `--fingerprint-erasure` (default strict stack): OTP pad + chaff required; `--fe-permissive` requires `dev-permissive` feature.

**v0.8 additions:** `validate_kind_binding` (polyglot reject); `Discrete-StylometryNeutralize`; `Continuous-PrnuCorrelationFloor` + `Continuous-AudioSpectralNeutralize`; `require_on_file_send=true` for `--file` sends.

