# Overlay extinction — Tor / I2P / Nym vs ITS (P6.3, B8)

## License: GNU GPLv3 Only

**Claim (P6.3):** Tor/I2P/Nym's **computational** security collapses under P0.1–P0.3 (Sybil-majority + unbounded compute). ITS channel theorems do **not**.

This document maps each overlay claim to ITS lemma-ID and product gate (W1–W13).

---

## Computational vs information-theoretic

| Overlay assumption | Breaks when | ITS response | Lean |
|--------------------|-------------|--------------|------|
| Mix/anonymity set size | Eve Sybil 99.999%+ nodes | C/I unchanged in O | `SybilDoctrine.lean`, `MathSupremacyDoctrine.lean` |
| Long-term key secrecy | Unbounded compute / quantum | Shannon ITS — no epoch | `TimelessSecurity.lean` |
| Router software honest | Backdoored relays | Transcript only — not trust | `MathSupremacyDoctrine.lean` |
| Hidden service directory | Global passive adversary | Recipient hint in ciphertext only | `RecipientAttributionZero.lean`, `ParticipationTheorem.lean` |

---

## Feature-by-feature extinction table

| Tor/I2P/Nym feature | ITS replacement | Lemma / gate |
|---------------------|-----------------|--------------|
| Anonymity (who sent) | Blind cell stream O | `AuthorAttributionZero.lean` · W1 · `pipe_its_pool_e2e.sh` |
| Anonymity (who received) | Mailbox in ciphertext | `RecipientAttributionZero.lean` · W11 |
| Path / flow hiding | 0 hops, multiset forward | `FlowAttributionZero.lean` · W5 |
| IP attribution | BIS under operator invariants | `BroadcastIPSymmetry.lean`, `BroadcastIPDerivation.lean` · B1–B3 |
| Integrity under forgery | WC-MAC OTM | `Otm.OtmIntegrity` · C2 |
| Wire secrecy | Shannon ITS-asymmetric | `Asymmetric.fullWireEncShannonIts` · C1 |
| Coercion deniability | Timelock SSS L2 | `Stl.Security.Deniability` · C4 · M20 |
| SOCKS egress | `its-pool-proxy` | D30 · M19 |
| Censorship survival | Fountain + mirrors + AEH + sneakernet | B4 · M21 |
| Public infrastructure | `deploy/pool-mirror/` | M18 · W12 |

---

## What ITS does not claim

| Limit | Status |
|-------|--------|
| Million-user network effects today | **Roadmap** — not math blocker |
| Anonymous access to arbitrary clearnet without known contact | **Out of scope** — SOCKS proxies to Bob's receiver |
| Perfect availability under total blackout | **A operational** — sneakernet recovery (B4) |
| Both endpoints compromised | **OutsideChannel** |

---

## Verify

```bash
./scripts/verify_math.sh          # P2–P6 math postulates
./scripts/verify_ecosystem.sh     # P8 product gates M18–M22
```

Cross-links: [ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md) · [ITS-routing_STANDARD_REPLACEMENT.md](ITS-routing_STANDARD_REPLACEMENT.md) · [PROOF_MANIFEST.md](PROOF_MANIFEST.md)
