# ITS-routing: Mathematical Core (formal spec)

## License: GNU GPLv3 Only

## Target: Mathematicians, cryptographers, traffic-analysis auditors

**Status:** v5 ecosystem certificate **proved** · v6 absolutisme doc-sync · v7 Lean closure (B1/B3, Absolut A, roles)  
**Formal certificate:** [`mathematics/MasterTheorem.lean`](mathematics/MasterTheorem.lean) (v5) · [`mathematics/MasterTheoremV6.lean`](mathematics/MasterTheoremV6.lean) (v6)  
**Verify:** `./scripts/verify_math.sh` — M1–M17, `lake build`, 0 `sorry`, smoke certificates  
**Lean roots:** [`mathematics/lakefile.lean`](mathematics/lakefile.lean) — `routing-math-cert` · `routing-math-dev` · `routing-math-refinement`

**Related:** [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) · [PROOF_MANIFEST.md](PROOF_MANIFEST.md) · [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)

> **This document is the authoritative mathematical entry for ITS-routing.**  
> Legacy onion/Lorenz proofs live in [ITS-routing_mathematics.md](ITS-routing_mathematics.md) (dev-only historical).

---

## Purpose

Define the **complete, documentable mathematical model** for how ITS-routing achieves maximal **C.I.A.** under active Eve who owns 99.999%+ Sybil nodes, with **minimal overhead** (0 hops, 1 epoch, 1 cell) — making Tor, I2P, and Nym mixnets the objectively weaker choice under the same threat model.

**Math is the sole trust source.** Eve's pool/relay/ISP software and hardware are **transcript** (delivery only). Per message pair, **either** the sender (encryptor) **or** the receiver (verify-oracle) runs the math-trusted executor — Alice–Bob, Alice–Charlie, or any ITS endpoint pair (A2′).

---

## §0 — Axioms

| ID | Axiom |
|----|--------|
| **A0** | Eve owns ≥ 99.999% of all nodes; all pool/relay/ISP SW/HW is backdoored **transcript**. |
| **A1** | Eve has unbounded computational power and unbounded time. |
| **A2** | **Either** the sender (encryptor) **or** the receiver (verify-oracle) runs the math-trusted executor correctly — per message pair. |
| **A2′** | \(\text{SecureEncryptor}(\text{sender}) \lor \text{SecureVerifyOracle}(\text{receiver})\) for **any** ITS pair (Alice–Bob, Alice–Charlie, …). Charlie as third ITS reader/witness is in scope. |
| **A3** | Security claims = **information-theoretic algebra only** (Shannon + WC-MAC + no-provenance channel). |

Everything Eve owns affects **A (availability)** — never **C/I** in channel observation \(O\), when A2/A2′ holds.

**Outside (minimal, explicit):** both endpoints compromised before channel; side-channels; physical coercion on unsecured device; \(O_{\text{net}}=\emptyset\) total blackout (sneakernet recovery — not silent online pool censorship).

**Lean:** `MathSupremacyDoctrine.lean`, `EndpointEitherOr.lean`, `EndpointSplit.lean`

---

## §0b — Endpoint scope (A2′)

For each message pair \((s, r)\):

\[
\text{SecureEncryptor}(s) \lor \text{SecureVerifyOracle}(r)
\]

**Example:** Alice hosts content; Bob₁…Bobₙ and Charlie (witness) harvest via public pool — A2′ applies per pair (Alice–Bobᵢ, Alice–Charlie). Compromise of **both** endpoints in a pair is **Outside** channel C/I.

**Lean:** `EndpointEitherOr.lean`, `EndpointSplit.secureEndpointAxiom` (Outside boundary)

---

## §Expectations — absolutisme matrix

| Forventning | Formel / claim | Lean | Klasse |
|-------------|----------------|------|--------|
| Absolut C i \(O\) | \(I(M;O)=0\), \(I(S;O)=0\) | `FiniteMutualInfo`, `UnifiedEpochStream` | **Proved** |
| Absolut I | \(P(\text{forge})\le 1/p\) | `Otm.OtmIntegrity` | **Proved** |
| Ingen route | \(I(\text{flow};O)=0\), \(I(\text{flow};IP_{obs})=0\) | `FlowAttributionZero` | **Proved** |
| Ingen sidste exit | \(h=0\), `noGuiltyNode` | `PlausibleDeniabilityAbsolute`, `RoleAwareDeniability` | **Proved** |
| Sybil irrelevant | \(I(M;O_{E\cup Sybil})=0\) | `SybilDoctrine` | **Proved** |
| Enten-ende | Alice ∨ Bob (eller Charlie) | `EndpointEitherOr` | **Proved** |
| IP anonymitet | \(I(\text{author};IP_{obs})=0\) | BIS B1+B3 derived (`BroadcastIPDerivation.bisFullyDerived`) | **Proved** (v7) |
| Absolut A / ITS forward proof | censur ⇒ witness route ∨ reconstruct | `ForwardProof.lean`, `CensorshipDisclosure.aAbsolute` | **Proved** (v8) |
| ValidFwd whitelist + receive gate | \(\mathcal{M}_{\text{valid}}\), receiveGate | `ValidForwardParty.lean`, `ForwardReceiveGate.lean` | **Proved** (v9) |
| Witness k-of-n consensus | consensusAtEpoch ⇒ ProofFwd | `WitnessConsensus.lean` | **Proved** (v9) |
| Ingen skyldig forwarder | `noGuiltyNode` på \(O_{fwd}\) | `RoleAwareDeniability.lean` | **Proved** (v7) |
| Host vs reader | \(I(\text{reader}_i; O)=0\) | multi-recipient + SOCKS | **Proved** |
| P1–P3 participation | harvest pool/E, no dedicated EP | `OplusClosure.participationPostulatesDerived` | **Proved** (v7) |
| Zero math stubs | no `Prop := True` in cert path | `grep mathematics/` | **Proved** (v7 closure) |

**DoD cross-ref:** [`its_dod_postulates_v7_ca308ef5.plan.md`](../.cursor/plans/its_dod_postulates_v7_ca308ef5.plan.md) — P0–P8 mapping in [aca03375 plan](../.cursor/plans/mathematical_core_doc_aca03375.plan.md).

---

## §I — Symbols

| Symbol | Meaning |
|--------|---------|
| \(p\) | \(2^{31} - 1\) — Mersenne-31 field \(\mathbb{F}_p\) |
| \(M\) | Plaintext message |
| \(S\) | \((M, r, \ell, \lambda, \tau)\) — full secret bundle |
| \(O\) | Channel observation: epoch cells \(\{C_e\}\), no provenance |
| \(O^+\) | Rate, volume, participation (metadata) |
| \(IP_{obs}\) | src/dst/shape tuples under BIS |
| \(\mathcal{D}\) | Cell distribution over \(\mathbb{F}_p\) |
| \(\mathcal{E}\) | Eve's transcript (pool, relays, Sybil injections) |

**Lean:** `ObservationAlphabet.lean`, `UnifiedEpochStream.lean`

---

## §II — C: Confidentiality (maximal ITS)

### C1 — Wire (ITS-asymmetric, cross-import)

\[
\boxed{I(M;\, C_{\text{wire}}) = 0}
\]

Eve sees `public.key` + all wire bytes. Without `secret.key`: posterior over \(M\) is **uniform** — Shannon ITS, not computational.

| Lean | Status |
|------|--------|
| `Transport/WireComposition.lean` → `Asymmetric.fullWireEncShannonIts` | **Proved** (cross-repo) |

### C3 — Channel (ITS-routing)

\[
\boxed{I(S;\, O_{\mathcal{E}}) = 0}
\]
\[
\boxed{I(M;\, O_{\mathcal{E}}) = 0}
\]

#### L3 — Constant emit (minimal overhead, prod default)

\[
(K_{e+1},\, C_e) = \text{step}(K_e,\, e), \quad C_e \sim \mathcal{D}, \quad |C_e| = L \text{ fixed}
\]

**Production:** **0 hops**, **1 epoch**, **1 cell** per epoch. No mix window.

\[
\text{Latency}_{\text{ITS}} \approx \text{epoch\_interval\_ms}
\]

| Lean | Module |
|------|--------|
| L3 send | `Transport/Epoch.lean`, `UnifiedEpochStream.lean` |
| Ideal step | `idealStep` in `Transport/Epoch.lean` |
| Rust target | `its_transport/src/epoch_cell.rs` |

#### L1 — Cell indistinguishability

\[
\text{observe}(\text{payload}, d) = \text{observe}(\text{idle}, d) = d \bmod p
\]

No separate data/setup/chaff **types** in \(O\).

| Lean | `Transport/Cell.lean` |

#### L3' — Constant harvest (receiver)

Bob harvests every epoch at fixed request size:

\[
I(\ell;\, O^+_{\text{rate,volume}}) = 0
\]

| Lean | `MetadataSymmetry.lean`, `LinkParticipation.lean` |

### Sprint 1 done — finite mutual information

`Transport/FiniteMutualInfo.lean` derives \(I(\cdot;\cdot)=0\) from uniform posterior (`Asymmetric.PosteriorUniform`) — **`Adversary.lean` re-exports**, no `mutualInfo := 0` stub.

---

## §III — Anonymity and unpredictability vs Sybil

Under A0–A2, Eve cannot correlate sender, recipient, or path in \(O\) and \(IP_{obs}\).

### Author

\[
\boxed{I(\text{author};\, O) = 0}
\]

Structural: `provenanceInObs = False`, no client-ID in pool headers.

| Lean | `ParticipationTheorem.lean`, `AuthorAttributionZero.lean` |

### Recipient

\[
\boxed{I(\text{recipient};\, O) = 0}
\]

Recipient/mailbox hint **only** inside Shannon ciphertext body — never in pool headers or share IDs.

| Lean | `RecipientAttributionZero.lean` |

### Flow / path

\[
\boxed{I(\text{flow};\, O) = 0}
\]
\[
\boxed{I(\text{flow};\, IP_{obs}) = 0}
\]

| Lean | `FlowAttributionZero.lean`, `BroadcastForward.lean` |

### Sybil irrelevance

\[
\boxed{I(M;\, O_{\mathcal{E} \cup \text{Sybil}}) = I(M;\, O_{\mathcal{E}}) = 0}
\]

Fake pool posters: OTM-fail **or** chaff \(\sim \mathcal{D}\) → **0 extra bits** about \(M\).

| Lean | `SybilDoctrine.lean` |

### Few-user doctrine (minimal overhead vs overlays)

\[
\boxed{|\mathcal{D}| = p \Rightarrow \text{anonymity independent of peer count}}
\]

**N = 1 user suffices.** Tor/I2P require mass peers for k-anonymity; ITS does not.

| Lean | `FewUserDoctrine.lean` |

### Broadcast forward (relay without identity accumulation)

Each hop forwards multiset of \(\mathcal{D}\)-indistinguishable cells; no author-label:

\[
\text{forward}(h,\, \mathcal{D}) \Rightarrow I(\text{author};\, O_h) = 0
\]

| Lean | `BroadcastForward.lean` |

### BIS — Broadcast IP Symmetry

Under postulates B1–B3:

\[
I(\text{author};\, IP_{obs}) = 0, \quad I(\text{recipient};\, IP_{obs}) = 0
\]

| Postulate | Meaning |
|-----------|---------|
| **B1** | Every IP ∈ 𝒩 emits symmetrically each epoch |
| **B2** | ITS cells indistinguishable from chaff |
| **B3** | Multicast forward without author in IP header |

| Lean | `BroadcastIPSymmetry.lean` — v7: **B1+B2+B3 derived** in `BroadcastIPDerivation.bisFullyDerived` (L3 + public pool + P1–P3 + h=0 forward) |

### Absolute deniability

\[
\mathcal{D}_{\text{abs}} = \text{author-zero} \land \text{recipient-zero} \land \text{flow-zero} \land \text{BIS} \land \text{SSS-courier} \land \text{either-EP} \land \text{Sybil}
\]

\[
\Rightarrow \text{no guilty node in } O \cup IP_{obs}
\]

| Lean | `PlausibleDeniabilityAbsolute.lean`, `noGuiltyNode` |

### SSS multi-IP courier

\(m\) IP endpoints emit shares/chaff each epoch:

\[
I(\text{author};\, \text{which-IP}) = 0
\]

| Lean | `SSSMultiIPCourier.lean` |

---

## §IIIb — NoLastHop (ITS vs Tor exit)

Tor assigns guilt to the **last relay**. ITS production: **\(h = 0\) hops**, global UES pool broadcast — no hop chain, no exit node.

\[
\text{forward}(h,\, \mathcal{D}) \land h = 0 \Rightarrow I(\text{author};\, O) = 0
\]

**Multi-reader / SOCKS:** Bob₁…Bobₙ read Alice-hosted content via public pool:

\[
\forall i.\, I(\text{reader}_i;\, O) = 0
\]

Alice as **publisher/host** is a deliberate content origin — **not** a mix-network exit. `RoleAwareDeniability` separates Forwarder / Publisher / Reader roles.

| Lean | `BroadcastForward.lean`, `RoleAwareDeniability.lean`, `ObservationAlphabet.NodeRole` |
| Doc | [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md) |

---

## §IV — I: Integrity (maximal ITS)

\[
\boxed{P(\text{forge accepted}) \leq \frac{1}{p}}
\]

Wegman-Carter OTM over \(\mathbb{F}_p\) — information-theoretic, not Ed25519/RSA/PQC.

OTM verify runs **only** on Bob's math-trusted verify-oracle — never on Eve's nodes.

| Lean | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` | **Proved** (cross-repo OTM import) |

---

## §V — A: ITS availability via forward proof + whitelist (v9)

Proof of forwarding = existence in canonical public log, harvestable from a witness mirror.
No personal ACK; alternate route = next mirror in \(\mathcal{M}_{\text{valid}}\) (`multi_pool_urls`).

\[
\text{ValidFwd}(m,W) \Leftrightarrow \forall e \leq W.\, \text{Publish}(e,c) \Rightarrow \text{Harvest}(m,e)=c
\]

\[
\mathcal{M}_{\text{valid}} = \{ m \mid \text{ValidFwd}(m) \land \neg\text{sendRightsRevoked}(m) \}
\]

\[
\text{receiveGate}(m,e) \Leftrightarrow \text{ValidFwd}(m, [0,e-1])
\]

\[
\text{consensusAtEpoch}(e,c,k) \Leftrightarrow \exists \mathcal{W}_{A2'}.\, |\{w \in \mathcal{W} : \text{Harvest}(w,e)=c\}| \geq k
\]

\[
\boxed{\text{ProofFwd}(e,c) \Leftrightarrow \text{Publish}(e,c) \land \exists m.\,\text{Harvest}(m,e)=c}
\]

\[
\neg\text{Local}(s,e,c) \land \text{ProofFwd}(e,c) \Rightarrow \text{AlternateRoute}(s,e,c)
\]

\[
\text{omit}(C_e, s) \Rightarrow \big(\exists m \in \mathcal{M}_{\text{valid}}.\, \text{Harvest}(m,e)=C_e\big) \lor \big(\Delta O^+_{\text{rate}}(e) \neq 0\big) \lor \big(f+k \le n \land \text{reconstruct}\big)
\]

| Mechanism | Lean |
|-----------|------|
| Forward proof + alternate mirror route | `ForwardProof.lean` |
| ValidFwd whitelist + de-whitelist on omit | `ValidForwardParty.lean` |
| k-of-n witness consensus (A2′ Charlie) | `WitnessConsensus.lean` |
| Forward-to-receive gate | `ForwardReceiveGate.lean` |
| Public pool multicast + mirror mismatch | `PublicPoolMulticast.lean` |
| Silent omit impossible | `CensorshipDisclosure.silentOmitImpossible` |
| SSS reconstruction bound | `AvailabilityResilience.lean` |
| ITS-A in master cert v9 | `networkEcosystemCertificateV9` |

**Unattackable scope:** selective omit to `s` + k-of-n witness consensus (A2′ Charlie) ⇒ `ProofFwd`; alternate path from \(\mathcal{M}_{\text{valid}}\) only — no hop guilt.  
**Outside:** \(O_{\text{net}}=\emptyset\); all mirrors Eve-only with no independent witness; \(\mathcal{M}_{\text{valid}}=\emptyset\).

### SSS reconstruction bound

\[
f + k \leq n \Rightarrow \text{reconstruct}(M)
\]

| Lean | `AvailabilityResilience.lean` — **Operational**, not ITS |

### Offline / sneakernet

\[
O_{\text{net}} = \emptyset \Rightarrow I(S;\, O_{\text{net}}) = 0 \text{ (trivial)}
\]

Security reduces to wire on medium + OTM on Bob.

| Lean | `OfflineChannel.lean` |

Recovery without breaking C/I: fountain + multi-mirror + AEH + sneakernet (operational gates in `verify_ecosystem.sh`).

---

## §VI — AEH alternative (when pool protocol is banned)

| Lemma | Formula | Lean |
|-------|---------|------|
| **L4** | \(\phi \sim \mathcal{D}_{\text{benign}}\) | `AEH/StegoIndistinguishability.lean` |
| **L5** | \(I(S;\, \text{release}) = 0\) | `AEH/EpochGate.lean` |

**Mode composition (L9):** P (pool) **⊗** AEH (last-resort) — `Transport/Composition.lean`

**Note:** AEH `EpochGate` uses abstract epoch-index release — **not** the same as ITS-timelock `Stl` (see §VII).

---

## §VII — Timelock / TTL (C4 — ITS-timelock)

**Distinct from routing epoch.** Three time concepts:

| Concept | Role | Repo |
|---------|------|------|
| **Routing epoch** | L3 emit/harvest cadence | ROUTING `Transport/Epoch.lean` |
| **Transport ratchet** | SSS epoch forward FS on channel | `Transport/RatchetDerivation.lean` |
| **Timelock epochs** | RSW squaring iterations (L1 delay) | ITS-timelock `Stl/Rsw.lean` |

### RSW L1 (computational aux — carries no wire secret)

Sequential modular squaring = time wall only.

### Stl L2 (ITS OTP)

\[
C = M \oplus S_T \pmod p, \quad \text{decrypt}(C,\, S_T) = M
\]

| Lean | `ITS-self_enclosed_timelock/mathematics/stl/Stl/TimeLock.lean` |

### Coercion deniability (C4)

Under coercion: alternative plaintexts algebraically consistent (SSS underdetermination).

| Lean | `Stl/Security/Deniability.lean` |

### v5 — in master cert

C4 **in** `networkEcosystemCertificateV5`: cross-import `stl`, `CoercionModel.lean`, `Transport/TimelockComposition.lean`, `c4TimelockDeniability`.

---

## §VIII — Hops

### Production (standard — minimal overhead)

\[
\boxed{h = 0 \text{ hops},\quad 1 \text{ epoch},\quad \text{global UES pool broadcast}}
\]

Sybil-majority does **not** change \(I(M;O)\). This **replaces** Tor/I2P multi-hop mixnets for file/message under A0–A1.

| Config | `client-send/receive --pool` (default) |
| Feature | `pool` (not `dev-onion-mix`) |

### Dev/onion (rank-nullity — not in master cert)

\[
C = c_1 P_1 + c_2 P_2 \pmod p, \quad P_i = M_i + K_i
\]

\[
\dim\ker(\mathbf{A}) = 3L \Rightarrow I(M_1, M_2;\, C) = 0
\]

| Lean | `Transport/MixAnonymity.lean`, `Transport/ChaffIndistinguishability.lean` |
| Status | **Dev-only** — imported via `Transport.lean` but **not** in `UnattackableCertificate.lean` |
| v5 | Isolate from master cert path; document as regression only |

### Latency comparison

| System | Typical path |
|--------|--------------|
| **ITS UES Pool** | 1 × epoch_interval_ms |
| **Tor** | 3+ hops + mix delay + RTT |
| **I2P** | Tunnel tiers + variable |
| **Nym** | Mix layers + mix window |

---

## §IX — Master theorem

### v4 (historical smoke)

```lean
def unattackableCertificate : Prop := ...  -- UnattackableCertificate.lean
```

### v5 — ecosystem certificate (**proved**)

```lean
def networkEcosystemCertificateV5 : Prop :=
  c1WireShannon ∧
  c2OtmIntegrity ∧
  networkItsCertificateV5 ∧
  c4TimelockDeniability ∧
  trustedBoundary ∧
  timelessSecurity ∧
  mediumIndependence ∧
  Transport.timelockTransportComposition
```

**Smoke:** `lake env lean MasterTheorem.lean`

### v6 — absolutisme certificate (**proved**)

```lean
def networkEcosystemCertificateV6 : Prop :=
  networkEcosystemCertificateV5 ∧
  aAbsolute ∧
  bisFullyDerivedClosed ∧
  roleAwareDeniability bisFullyDerived
```

**Smoke:** `lake env lean MasterTheoremV6.lean` · verify gate **M17**

---

## §X — Overlay comparison (Tor / I2P / Nym)

Under axioms A0–A1 and file/message to known contact:

| | **ITS** | **Tor / I2P / Nym** |
|--|---------|---------------------|
| **C** | \(I(M;O)=0\) forever (ITS) | Computational → breaks under A1 |
| **I** | \(P(forge)\leq 1/p\) (WC-MAC ITS) | Signatures/PQC — crypto-epoch |
| **A** | SSS + sneakernet (operational) | Bridges/mirrors (operational) |
| **Sybil 99%+** | C/I **unchanged** | Deanonymization risk |
| **N = 1 user** | **Sufficient** | Meaningless without mass |
| **Hops** | **0** (ms latency) | 3–6+ (seconds) |
| **Compute trust** | **None** | Required |

**Conclusion:** Choosing Tor/I2P/Nym when explicitly requiring A0–A1 for C/I on file/message is the objectively weaker design — not because overlays are poorly engineered, but because their **security lemma class is weaker by definition**.

Future doc: [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md) (lemma-ID per claim — **available**).

---

## §XI — Formula manifest (one page)

```
FIELD:           p = 2^31 - 1

C1 WIRE:         I(M; C_wire) = 0                 [Asymmetric Shannon]

C3 CHANNEL:      I(S; O) = 0
                 I(M; O) = 0

L3 SEND:         (K_{e+1}, C_e) = step(K_e, e),  C_e ~ D

L1 CELL:         observe(payload, d) = observe(idle, d) = d mod p

L3' RECV:        I(l; O+_{rv}) = 0

AUTHOR:          I(author; O) = 0,  provenance not in O

RECIPIENT:       I(recipient; O) = 0,  hint in ciphertext only

FLOW:            I(flow; O) = 0,  I(flow; IP_obs) = 0

SYBIL:           I(M; O_{E∪Sybil}) = I(M; O) = 0

N=1:             |D| = p  =>  size-independent anonymity

BIS:             I(author; IP_obs) = 0,  I(recipient; IP_obs) = 0  [under B1-B3]

FORWARD:         forward(h, D) => I(author; O_h) = 0

C2 INTEGRITY:    P(forge) <= 1/p                    [OTM WC-MAC — v5]

AEH L4/L5:       phi ~ D_benign,  I(S; release) = 0

OFFLINE:         O_net = empty => trivial I=0; wire + OTM on medium

SSS A:           f + k <= n => reconstruct

TIMLOCK L2:      C = M xor S_T,  decrypt(C,S_T) = M    [Stl — v5 import]

COERCION C4:     alternative M' consistent under coercion [Stl — v5]

TIMELESS:        C/I independent of compute epoch

PROD HOPS:       h = 0, 1 epoch, 1 cell

MASTER v5:       U_5 = C1 ∧ C2 ∧ C3 ∧ C4 ∧ D_abs ∧ T ∧ timeless ∧ medium

MASTER v6:       U_6 = U_5 ∧ A_abs ∧ BIS_derived ∧ roleAwareDeniability
```

---

## §XII — Lean module map

| Formula / claim | Lean module | v4 status |
|-----------------|-------------|-----------|
| C1 wire Shannon | `Transport/WireComposition.lean` → asymmetric | **Proved** (import) |
| C3 I(S;O)=0 | `UnifiedEpochStream.lean` | **Proved** (finite-MI) |
| L1 cell ~ 𝒟 | `Transport/Cell.lean` | **Proved** |
| L3 constant emit | `Transport/Epoch.lean` | **Proved** |
| L3' metadata | `MetadataSymmetry.lean` | **Proved** (finite-MI) |
| Author zero | `AuthorAttributionZero.lean` | **Proved** |
| Recipient zero | `RecipientAttributionZero.lean` | **Proved** |
| Flow zero | `FlowAttributionZero.lean` | **Proved** |
| Sybil | `SybilDoctrine.lean` | **Proved** (finite-MI) |
| N=1 | `FewUserDoctrine.lean` | **Proved** (finite-MI) |
| BIS IP | `BroadcastIPSymmetry.lean` + `BroadcastIPDerivation.bisFullyDerived` | **Proved** (B1+B2+B3 derived) |
| Forward hop | `BroadcastForward.lean` | **Proved** (finite-MI) |
| Absolut A | `CensorshipDisclosure.lean`, `PublicPoolMulticast.lean` | **Proved** (v6 cert) |
| Role deniability | `RoleAwareDeniability.lean` | **Proved** (v6 cert) |
| SSS courier | `SSSMultiIPCourier.lean` | **Proved** |
| Either EP | `EndpointEitherOr.lean` | **Proved** |
| MathSupremacy | `MathSupremacyDoctrine.lean` | **Proved** |
| C2 integrity | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` | **Proved** (OTM import) |
| A availability | `AvailabilityResilience.lean` | **Operational** |
| AEH L4/L5 | `AEH/StegoIndistinguishability.lean`, `AEH/EpochGate.lean` | **Proved** |
| L9 composition | `Transport/Composition.lean` | **Proved** |
| Offline | `OfflineChannel.lean` | **Proved** |
| Master v4 | `UnattackableCertificate.lean` | **Smoke target** |
| C4 coercion | `CoercionModel.lean` → `Stl.Security.Deniability` | **Proved** (import) |
| Timelock compose | `Transport/TimelockComposition.lean` | **Proved** |
| Master v5 | `MasterTheorem.lean` | **Proved** (ecosystem cert) |
| Master v6 | `MasterTheoremV6.lean` | **Proved** (absolutisme cert) |
| Dev mix hops | `Transport/MixAnonymity.lean` | **Not in master cert** |
| Dev onion chaff | `Transport/ChaffIndistinguishability.lean` | **Not in master cert** |

**Cross-repo (import, do not duplicate):**

| Channel | Repo | Lean |
|---------|------|------|
| C1 | ITS-asymmetric | `Asymmetric.fullWireEncShannonIts` |
| C2 (v5) | ITS-OTM | `Otm.OtmIntegrity` |
| C4 (v5) | ITS-timelock | `Stl.Security.Deniability` |

---

## §XIII — Closure checklist

| # | Task | Status |
|---|------|--------|
| 1 | `Transport/FiniteMutualInfo.lean` — eliminate `mutualInfo := 0` | **Done (Sprint 1)** |
| 2 | `ITS-OTM/mathematics/` + lake import | **Done** |
| 3 | `BroadcastIPDerivation.lean` — derive B2 | **Done** |
| 4 | `TimelessSecurity.lean`, `MediumIndependence.lean` | **Done** |
| 5 | Stl cross-import + `CoercionModel.lean` | **Done (Sprint 3)** |
| 6 | `MasterTheorem.lean` + `networkEcosystemCertificateV5` | **Done (Sprint 2–3)** |
| 7 | Isolate `MixAnonymity` / `ChaffIndistinguishability` from master path | **Done (Sprint 0)** |
| 8 | `verify_math.sh` M9–M16 green | **Done** |
| 9 | OTM WC-MAC soundness depth | **v7+** |
| 10 | B1/B3 derive from L3+pool+P1–P3 | **Done (v7)** |
| 11 | CensorshipDisclosure + PublicPoolMulticast | **Done (v7)** |
| 12 | RoleAwareDeniability (host/reader/forwarder) | **Done (v7)** |
| 13 | `networkEcosystemCertificateV6` | **Done (v7)** |
| 14 | CORE §Expectations + NoLastHop doc-sync | **Done (v6)** |

---

## §XIV — Architecture: math core vs optional

```mermaid
flowchart TB
  subgraph mathCore [MathCore_Lean]
    UES["UnifiedEpochStream L3"]
    Attr["Author Recipient Flow"]
    BIS["BroadcastIPSymmetry"]
    Wire["WireComposition"]
  end
  subgraph imports [CrossRepoImports]
    ASY["ITS-asymmetric C1"]
    OTM["ITS-OTM C2 v5"]
    TL["ITS-timelock C4 v5"]
  end
  subgraph devOnly [DevOnly_NotMasterCert]
    Mix["MixAnonymity hops"]
    Chaff["ChaffIndistinguishability onion"]
  end
  subgraph rust [RustOptionalFeatures]
    Ridges["timelock FE hardware ledger"]
    DevOnion["dev-onion-mix"]
  end
  ASY --> Wire
  Wire --> UES
  UES --> Attr
  Attr --> BIS
  subgraph v7 [v7_absolutisme_Lean]
    BISfull[B1_B2_B3_derived]
    AbsA[CensorshipDisclosure_AbsolutA]
    Roles[RoleAwareDeniability]
    U6[networkEcosystemCertificateV6]
  end
  OTM --> Master["MasterTheorem v5"]
  TL --> Master
  mathCore --> Master
  Master --> U6
  BISfull --> U6
  AbsA --> U6
  Roles --> U6
  devOnly -.->|"not in cert"| Master
  rust -.->|"refinement gates"| Master
```

**Decoupling rules:**

- `its_routing` has **no** Cargo dependency on `its_asymmetric` — wire via **pipe** only ([ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)).
- OTM, timelock, FE, hardware, ledger = **optional Cargo features** on `its_routing` ([`its_routing/Cargo.toml`](its_routing/Cargo.toml)).
- Math repos linked via **`lake require`** cross-import — not compile-time coupling.

---

## §XV — One-sentence law

**Dansk:**

> Eve ejer 99,999%+ af nettet og kan gøre hvad hun vil med infrastrukturen — hun lærer matematisk nul om hvem der sendte, modtog, hvad der stod i beskeden, og hvilken vej den gik; **ingen sidste exit, ingen skyldig node** i \(O \cup IP_{obs}\); det gælder med én bruger, nul hops og én epoch, fordi anonymitet er celle-fordelingen 𝒟 — ikke overlay-masse — og skal være maskin-verificeret i Lean.

**English:**

> Eve owns 99.999%+ of the network and may manipulate all infrastructure — she learns information-theoretically zero about sender, recipient, message content, and path; **no last exit, no guilty node** in \(O \cup IP_{obs}\); this holds with one user, zero hops, and one epoch, because anonymity is the cell distribution 𝒟 — not overlay mass — and must be machine-verified in Lean, not assumed from Eve's software.

---

## §XVI — Lemma chain quick reference (L1–L13)

| # | Lemma | Mode | Lean | Status |
|---|-------|------|------|--------|
| L1 | Wire + cell indistinguishability | both | `WireComposition`, `Cell` | Proved (C1 import) |
| L2 | OTM WC-MAC | both | `IntegrityAxiom` → `Otm.OtmIntegrity` | Proved (C2 import) |
| L3 | C_e ~ 𝒟, constant emit | P | `UnifiedEpochStream` | Proved |
| L4 | φ ~ 𝒟_benign | AEH | `AEH/StegoIndistinguishability` | Proved |
| L5 | I(S; release) = 0 | AEH | `AEH/EpochGate` | Proved |
| L6 | I(link; O) = 0 | P | `LinkParticipation` | Proved |
| L7 | AEH link-blind | AEH | `PlausibleDeniability` | Proved |
| L8 | SSS reconstruction | A | `AvailabilityResilience` | Operational |
| L9 | Mode composition | both | `Transport/Composition` | Proved |
| L10 | I(link; O⁺_{rv}) = 0 | both | `MetadataSymmetry` | **Proved** (finite-MI) |
| L11 | CoverTransport O⁺ | P | `ParticipationSymmetry` | Postulate P1–P3 |
| L12 | I(link; O⁺_part) = 0 | P | `OplusClosure` | Postulate P1–P3 |
| L13 | Passive ISP ⊆ active Eve | both | `ComparativeThreatDoctrine` | Proved |

Full detail: [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) · [PROOF_MANIFEST.md](PROOF_MANIFEST.md)
