# Epoch Pool Transport — Mathematical Specification, Postulates & Proofs

## License: GNU GPLv3 Only
## Target: Mathematicians, Cryptographers & Independent Reviewers

*(Implementation repository name: ITS-routing. This document uses standard cryptographic language only; CLI names and Lean modules appear in appendices.)*

---

## Purpose

This document specifies **epoch pool transport**: how Shannon-perfect-secrecy **wire ciphertexts** (from static broadcast encryption) are published as **epoch cells** on a hostile pool channel. An adversary who observes the full channel view — but not the decryptor's local secrets — cannot determine plaintext $M$.

Security rests on **three separable layers**:

1. **Wire layer (imported):** $I(M; C_{\mathrm{wire}}) = 0$ — proved in [ITS-asymmetric](https://github.com/0x1F980/ITS-ASYMMETRIC/blob/main/ITS-asymmetric_mathematics.md) (Theorem 7.1 there).
2. **Channel layer (this repo):** $I(M; O) = 0$ where $O$ is the sequence of published epoch cells with **no provenance** (Theorem 7.1 below).
3. **Availability layer (operational, not Shannon):** selective omit by hostile mirrors is detectable; harvest reroutes via valid-forward whitelist and witness consensus (§8 — not a secrecy claim).

**Production model:** $h = 0$ hops, one cell per epoch, global pool broadcast (Postulate P6).

**Reviewer task:** Read **§0.1** (worked example), then postulates and theorems, then Appendices A–B to **confirm** or **reject** that the implementation matches this specification.

> **Convention.** §2–§12 use mathematical symbols only. Rust/CLI identifiers and Lean theorem names appear **only** in appendices and *implementation correspondence* lines.

**Deep proof map:** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) (Lean certificate M1–M26, extended scenarios).

---

## 0. Notation

| Symbol | Meaning |
|--------|---------|
| $p$ | $2^{31}-1$ (Mersenne prime; field modulus $\mathbb{F}_p$) |
| $\mathbb{F}_p$ | Finite field $\mathbb{Z}/p\mathbb{Z}$ |
| $M$ | Plaintext message (byte string) |
| $C_{\mathrm{wire}}$ | Wire ciphertext from broadcast encrypt ($\mathsf{pk}$ + per-message bytes) |
| $e$ | Epoch index ($e \in \mathbb{N}$) |
| $C_e$ | Published cell at epoch $e$ (fixed length $L$) |
| $O$ | Adversary channel view: $(C_0, C_1, \ldots, C_{e_{\max}})$ — **no** sender/recipient labels |
| $O^+$ | Rate/volume metadata (harvest cadence); treated separately in §9 |
| $\mathcal{D}$ | Cell draw distribution over $\mathbb{F}_p$ ($|\mathrm{supp}(\mathcal{D})| = p$) |
| $\mathsf{pk}$, $\mathsf{sk}$ | Public / secret key (ITS-asymmetric; $\mathsf{sk}$ never on channel) |
| $\rho$ | Secret MAC root (never on channel) |
| $\mathcal{M}_{\mathrm{valid}}$ | Mirror whitelist: parties satisfying valid-forward |
| $K_e$ | Epoch ratchet state (local; not in $O$) |
| $\text{step}$ | Epoch update: $(K_{e+1}, C_e) = \text{step}(K_e, e)$ |

---

## 0.1 Worked example (read this first)

**Setup:** $p = 2{,}147{,}483{,}647$. Alice encrypts one byte $M[0]=65$ to Bob's $\mathsf{pk}$ (wire layer — see ITS-asymmetric §0.1: published $B[0]=9$, $\sigma[0]=300$, $\beta[0]=5900$, tag $\tau[0]$). Alice embeds the wire blob in epoch $e=7$ cell $C_7$ and publishes to the pool. Eve controls 99.999% of mirrors but **not** Bob's $\mathsf{sk}$.

### Wire inside cell (imported)

From ITS-asymmetric §0.1, Eve's wire-only view for byte 0:

| Published (wire) | Value |
|------------------|-------|
| $B[0]$ | $9$ |
| $\sigma[0]$ | $300$ |
| $\beta[0]$ | $5900$ |

Without $\mathsf{sk}$, $\rho$, mask keys: all $256$ plaintext bytes remain consistent — $I(M[0]; C_{\mathrm{wire}}) = 0$.

### Epoch publish (transport)

Ideal epoch step (production cadence):

$$C_e = \text{draw}(K_e, e) \in \mathbb{F}_p, \qquad K_{e+1} = \text{advance}(K_e, e).$$

Concrete toy draw at $e=7$ (Lean ideal model uses $C_e \equiv e \bmod p$ for schedule proof only):

| Epoch | Published cell $C_e$ (tag in $\mathbb{F}_p$) |
|-------|-----------------------------------------------|
| $6$ | (idle/chaff draw) |
| $7$ | encodes wire payload for $M[0]=65$ |
| $8$ | (idle/chaff draw) |

**Eve's channel view** $O$:

$$O = (\ldots, C_6, C_7, C_8, \ldots)$$

**Not in $O$:** author identity, recipient identity, hop path, $\mathsf{sk}$, $\rho$, mask keys, cell label {payload vs idle}.

### Cell indistinguishability (Postulate P4)

For any draw $d \in \mathbb{F}_p$:

$$\mathrm{observe}(\text{payload}, d) = \mathrm{observe}(\text{idle}, d) = d \pmod p.$$

Eve cannot tell from $C_7$ alone whether epoch 7 carried message material or chaff — only that $C_7 \in \mathrm{supp}(\mathcal{D})$.

### Composition (Theorem 7.1 sketch)

Wire Shannon + uniform cell posterior $\Rightarrow$ no byte of $M$ is determined by $O$:

$$I(M; O) = 0.$$

Bob decrypts with $\mathsf{sk}$ on his trusted host (Postulate P2); Eve's mirrors never run verify with $\rho$.

### Availability aside (not Shannon)

If Eve's mirror drops $C_7$ at harvest, Bob may fetch the same epoch from another $m \in \mathcal{M}_{\mathrm{valid}}$ or witness consensus (§8). That restores **delivery**; it does not give Eve $M$.

*Implementation correspondence:* `UnifiedEpochStream`, `WireComposition` — Appendix B.

---

## Postulates

| ID | Postulate |
|----|-----------|
| **P0** | Field arithmetic in $\mathbb{F}_p$, $p = 2^{31}-1$. Byte wire arithmetic mod $256$ in imported wire spec. |
| **P1** | **Adversary Eve:** sees $\mathsf{pk}$, all wire bytes on the pool, full sequence $O$, and $O^+$ if published; not $\mathsf{sk}$, $\rho$, mask keys, or local ratchet $K_e$. Unbounded computation. |
| **P2** | **Endpoint trust (either-or):** per message pair $(s,r)$, the sender runs correct encrypt **or** the receiver runs correct decrypt/verify on a trusted host — not both required to fail. |
| **P3** | **No provenance in $O$:** pool cells carry no author ID, recipient ID, or route label in the adversary view. |
| **P4** | **Cell indistinguishability:** payload and idle cells share the same observation map $\mathrm{observe}(\cdot, d)$. |
| **P5** | **L3 cadence:** exactly one cell per epoch from each publisher; fixed cell size $L$. |
| **P6** | **Production topology:** $h = 0$ transport hops; global pool broadcast (no onion hop chain in cert path). |
| **P7** | **Wire import:** $I(M; C_{\mathrm{wire}}) = 0$ under ITS-asymmetric postulates (cross-repo). |
| **P8** | **Integrity:** Wegman–Carter verification uses secret $\rho$; without $\rho$, tags are non-informative for plaintext elimination in the wire proof. |
| **P9** | **Sybil cells:** injections from fake pool nodes either fail integrity or are drawn from $\mathcal{D}$ — zero extra Shannon bits about $M$. |
| **P10** | **Not claimed:** Shannon secrecy if **both** endpoints in a pair are compromised; side channels; physical coercion; $O_{\mathrm{net}} = \emptyset$ total blackout; k-anonymity from peer count. |

---

## 1. Proof map

| § | Content |
|---|---------|
| **0.1** | Worked example (wire + epoch) |
| 2–3 | Field, epoch step |
| 4–5 | Wire composition, adversary view |
| 6–7 | Channel theorems (Shannon) |
| 8 | Availability (valid-forward, witnesses) |
| 9 | Attribution bounds |
| 10–12 | Non-claims, comparison, checklist |
| **A–B** | Implementation / Lean audit |

---

## 2. Field (P0)

$$p = 2^{31} - 1 = 2147483647.$$

Cells and OTM tags live in $\mathbb{F}_p$. Wire bodies live mod $256$ (imported spec).

---

## 3. Epoch stream (P5, P6)

### 3.1 Step function

For epoch $e \ge 0$, local state $K_e$:

$$(K_{e+1}, C_e) = \text{step}(K_e, e), \qquad C_e \sim \mathcal{D}.$$

### 3.2 Observation alphabet

$$O = (C_0, C_1, \ldots, C_{e_{\max}}).$$

Provenance bits are excluded from $O$ (P3).

### 3.3 Production parameters

| Parameter | Production value |
|-----------|------------------|
| Hops $h$ | $0$ |
| Cells per epoch | $1$ |
| Mix window | none |

*Implementation correspondence:* `idealStep`, `defaultL3Send` — Appendix A.

---

## 4. Wire composition (P7)

Wire ciphertext is the payload inside selected epoch cells. Confidentiality **imports** the ITS-asymmetric theorem:

$$I(M; C_{\mathrm{wire}}) = 0.$$

**Lemma 4.1 (payload in channel).** If wire Shannon holds and cells are type-blind (P4), then wire entropy is not increased by pooling:

$$I(M; O) = 0 \quad \text{(Theorem 7.1).}$$

*Implementation correspondence:* cross-import `Asymmetric.fullWireEncShannonIts` — Appendix B.

---

## 5. Adversary view (P1, P3)

$$O = (C_0,\ldots,C_{e_{\max}}), \quad C_e \in \mathrm{supp}(\mathcal{D}).$$

**Published:** $\mathsf{pk}$, all wire bytes, all cells, optional $O^+$ statistics.

**Not published:** $\mathsf{sk}$, $\rho$, $(K_{1,i}, K_{2,i})$, ratchet seeds, author/recipient labels, decrypt shortcuts.

---

## 6. Integrity (P8)

### Theorem 6.1 (OTM forgery bound)

For Wegman–Carter tags over $\mathbb{F}_p$, an adversary without $\rho$ who submits a forged cell accepted by the verify-oracle satisfies:

$$P(\text{forge accepted}) \leq \frac{1}{p}.$$

**Proof:** Standard WC bound; verification runs only on the trusted receiver (P2). Machine-checked via OTM import — Appendix B.

### Worked numeric bound

$p = 2{,}147{,}483{,}647 \Rightarrow P \leq 4.66 \times 10^{-10}$ per attempt.

---

## 7. Channel perfect secrecy

### Definition 7.1

The transport has **Shannon perfect secrecy** on the pool channel if, for all messages $M$ in the finite message space:

$$I(M; O) = 0.$$

Equivalently: every consistent plaintext remains equally plausible given $(\mathsf{pk}, O)$.

### Theorem 7.1 (Eve cannot determine plaintext from channel)

Under postulates P0–P9 and imported wire Theorem 7.1 (ITS-asymmetric), Eve cannot compute $M$ from $(\mathsf{pk}, O)$.

**Proof sketch:** (1) Wire layer gives uniform posterior over $M$ given $C_{\mathrm{wire}}$. (2) P4 identifies payload/idle observations. (3) P9 absorbs Sybil injections without reducing entropy. (4) Finite mutual information machinery yields $I(M;O)=0$. $\square$

**Proof:** Machine-checked — `UnifiedEpochStream`, `FiniteMutualInfo`, `SybilDoctrine` (Appendix B).

### Theorem 7.2 (Sybil irrelevance)

Let $O_{\mathcal{E}}$ be Eve's honest transcript and $O_{\mathcal{E} \cup \mathrm{Sybil}}$ after Sybil flood. Then:

$$I(M; O_{\mathcal{E} \cup \mathrm{Sybil}}) = I(M; O_{\mathcal{E}}) = 0.$$

---

## 8. Availability (operational — not Shannon)

This section is **delivery**, not confidentiality.

### 8.1 Valid forward

Mirror $m$ is **valid** if it forwards every published cell in the canonical log up to window $W$:

$$\text{ValidFwd}(m, W) \Leftrightarrow \forall e \leq W.\; \text{Publish}(e, c) \Rightarrow \text{Harvest}(m, e) = c.$$

Selective omit removes $m$ from $\mathcal{M}_{\mathrm{valid}}$.

### 8.2 Worked example — mirror omit

Canonical log publishes $c_3$ at epoch $3$. Mirror Eve-A returns wrong/missing at $e=3$; mirror Bob-B returns $c_3$.

| Before omit | $\mathcal{M}_{\mathrm{valid}} = \{\text{Eve-A}, \text{Bob-B}\}$ |
| After omit at $e=3$ | $\mathcal{M}_{\mathrm{valid}} = \{\text{Bob-B}\}$ |

Receiver harvest gate reads only from $\mathcal{M}_{\mathrm{valid}}$ — alternate route without identifying a guilty hop ($h=0$).

### 8.3 Witness consensus

For witnesses $\mathcal{W}$, threshold $k$: if at least $k$ witnesses harvest $(e, c)$, then $\text{ProofFwd}(e,c)$ holds even if some mirrors omit.

**Not claimed as ITS:** availability does not imply $I(M;O)>0$ when omit occurs — omit affects delivery only.

---

## 9. Attribution bounds (structural)

Under P3 and production $h=0$:

$$I(\text{author}; O) = 0, \qquad I(\text{recipient}; O) = 0, \qquad I(\text{flow}; O) = 0.$$

Recipient hints appear **only** inside Shannon ciphertext bodies, never in pool headers.

IP observation symmetry (BIS) is derived under participation postulates — see MATHEMATICAL_CORE §III; confirm separately if IP threat model applies.

---

## 10. Comparison to mix-network confidentiality

| | Multi-hop mix (typical) | Epoch pool (this spec) |
|---|-------------------------|-------------------------|
| Secrecy basis | Often computational + crowd size | Shannon on wire + channel (P7, Thm 7.1) |
| Path in observation | Hop chain visible to design | $h=0$; no hop guilt |
| Peer count | k-anonymity needs many users | $|\mathcal{D}|=p$ — peer count independent (P10) |
| Eve owns 99%+ nodes | Often breaks anonymity | $I(M;O)=0$ unchanged (P9, Thm 7.2) |

---

## 11. Non-claims (P10)

Not provided: IND-CCA on pool; TLS/Web PKI replacement; Shannon delivery when $\mathcal{M}_{\mathrm{valid}}=\emptyset$; coercion deniability from $O$ alone; dev-only onion mix proofs (historical — not production cert path).

---

## 12. Review checklist

1. Accept postulates P0–P10?
2. Accept imported wire Theorem 7.1 from ITS-asymmetric?
3. Accept Theorem 7.1 (channel $I(M;O)=0$) — Appendix B?
4. Accept Theorem 6.1 (WC bound) on trusted verify-oracle?
5. Treat §8 availability as **non-Shannon** delivery only?
6. Verify Appendix A against `its_transport` / pool CLI?
7. Reject if you require provenance in $O$ or multi-hop mix in production path?

---

## 13. Summary identities

$$I(M; C_{\mathrm{wire}}) = 0 \quad \text{(imported wire)}$$

$$I(M; O) = 0 \quad \text{(channel, Theorem 7.1)}$$

$$P(\text{forge}) \leq \frac{1}{p}, \quad p = 2^{31}-1$$

$$(K_{e+1}, C_e) = \text{step}(K_e, e), \quad C_e \sim \mathcal{D}$$

---

## 14. Worked example (reference)

See **§0.1**: $M[0]=65$, wire $(9,300,5900)$, epoch cell $C_7$, Eve sees $O$ without $\mathsf{sk}$; $I(M;O)=0$; omit at mirror reroutes via $\mathcal{M}_{\mathrm{valid}}$.

---

## Appendix A — Implementation correspondence

**Not part of the mathematical definition.**

| Math | Implementation | Location |
|------|----------------|----------|
| $\text{step}$, $C_e$ | `epoch_cell`, ideal step | `its_transport/src/epoch_cell.rs` |
| L3 cadence | `default_l3_send` | `mathematics/Transport/Epoch.lean` |
| Cell observe | payload/idle blind | `mathematics/Transport/Cell.lean` |
| Wire embed | ITS-asymmetric encrypt | subprocess / `ITS_ASYMMETRIC_BIN` |
| Pool harvest | `receive_gate`, mirror list | `its_routing` pool CLI |
| $\mathcal{M}_{\mathrm{valid}}$ | valid-forward whitelist | config `multi_pool_urls` |
| OTM verify | `verify_public_otm_tag` | ITS-OTM integration |

On-disk config: `config.prod.toml`. Operator keys: [ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement) (out of scope here).

---

## Appendix B — Machine-checked proofs

| Claim | Proof location |
|-------|----------------|
| Theorem 7.1 (channel) | `UnifiedEpochStream.lean`, `FiniteMutualInfo.lean` |
| Theorem 7.2 (Sybil) | `SybilDoctrine.lean` |
| Wire import | `Transport/WireComposition.lean` → `Asymmetric.FiniteWireEnc` |
| Theorem 6.1 (OTM) | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` |
| P4 cell blind | `Transport/Cell.lean` |
| L3 epoch | `Transport/Epoch.lean` |
| Valid-forward / witness | `ValidForwardParty.lean`, `WitnessConsensus.lean` |
| Master certificate | `MasterTheoremV6.lean` |

Build: `./scripts/verify_math.sh` or `cd mathematics && lake build`.

Ecosystem definition of ITS: [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md). Extended scenarios: [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md).
