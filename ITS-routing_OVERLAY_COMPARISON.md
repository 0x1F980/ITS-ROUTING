# Overlay lemma mapping (P6.3)

Maps overlay features to ITS lemmas and gates. **Theorem-class comparison only.**

**Math spec:** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) · **Criteria table:** [ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md)

---

## Threat-model note (P6.3)

Under P0.1–P0.3 (Sybil-majority + unbounded compute), **computational** overlay anonymity claims and **Shannon** ITS channel claims are different lemma classes. This doc maps features to Lean modules — it does not rank engineering quality.

---

## Feature → lemma map

| Overlay feature | ITS mechanism | Lean / gate |
|-----------------|---------------|-------------|
| Sender anonymity | Blind cell stream $O$ | `AuthorAttributionZero.lean` · W1 · `pipe_its_pool_e2e.sh` |
| Recipient anonymity | Hint in ciphertext only | `RecipientAttributionZero.lean` · W11 |
| Path / flow hiding | $I(\text{flow};O)=0$, $h=0$ prod | `FlowAttributionZero.lean` · W5 |
| IP attribution | BIS (operator invariants) | `BroadcastIPSymmetry.lean`, `BroadcastIPDerivation.lean` |
| Integrity | WC-MAC OTM on A2′ EP | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` (**Import**) |
| Censorship | ITS-A: $\mathcal{M}_{\text{valid}}$ + witness | `ValidForwardParty.lean`, `WitnessConsensus.lean` · W8 |
| Offline | Sneakernet + wire on medium | `OfflineChannel.lean` · sneakernet pipes |
| Timelock / coercion | Stl L2 + C4 bundle | `CoercionModel.lean`, `TimelockComposition.lean` |

---

## Lemma-class differences (reference)

| Topic | ITS (ROUTING) | Typical overlay |
|-------|---------------|-----------------|
| C in $O$ | Shannon (Import C1 + Proved glue) | Computational anonymity set |
| Sybil (A0) | $I(M;O)$ unchanged in model | Model-dependent deanonymization |
| Latency (prod) | 1 × `epoch_interval_ms` | Tunnel / mix setup |
| Peer count | $|\mathcal{D}|=p$ | Often needs mass peers |

---

## Out of scope (explicit)

| Need | Status |
|------|--------|
| Anonymous clearnet to arbitrary sites | SOCKS to known Bob receiver only |
| Global hidden-service directory | Pairwise addressing (W11) |
| Browser-in-a-box UX | Operator CLI path: [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) |

---

## Cross-links

[ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md) · [ITS-routing_STANDARD_REPLACEMENT.md](ITS-routing_STANDARD_REPLACEMENT.md) · [ITS_MIGRATION_GUIDES.md](ITS_MIGRATION_GUIDES.md) · [PROOF_MANIFEST.md](PROOF_MANIFEST.md)
