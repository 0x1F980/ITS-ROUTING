# ITS-routing: Overlay lemma comparison

Technical comparison of **claim classes** under threat model A0–A3. Not a product recommendation.

**Authoritative math:** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) · **Threat model:** [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md)

---

## Criteria (W1–W13)

| # | Criterion | Typical overlay | ITS UES | Verify |
|---|-----------|-----------------|---------|--------|
| W1 | C/I under active Eve | Computational | Shannon $I(S;O)=0$ (Import C1 + Proved C3) | `UnifiedEpochStream.lean`; `pipe_its_pool_e2e.sh` |
| W2 | Sybil majority | Deanonymization risk (model-dependent) | $I(M;O)$ unchanged in Lean model | `SybilDoctrine.lean` |
| W3 | N=1 user | Often needs peer mass | `FewUserDoctrine` ($|\mathcal{D}|=p$) | `FewUserDoctrine.lean` |
| W4 | Unbounded compute (A1) | PQC migration pressure | MathSupremacy (ITS claims) | `MathSupremacyDoctrine.lean` |
| W5 | Latency | Multi-hop + mix window | 0 hops, 1 epoch | `epoch_interval_ms` in config |
| W6 | Idle leak | Model-dependent | L3 constant emit | `client.rs`; cover pipe |
| W7 | $O^+$ participation | Hop hiding | CoverTransport | `pipe_its_cover_harvest_e2e.sh` |
| W8 | Censorship recovery | Bridges / many nodes | ITS-A: ProofFwd + $\mathcal{M}_{\text{valid}}$ + witness k-of-n | `ForwardProof.lean`, `ValidForwardParty.lean`, `WitnessConsensus.lean`, `ForwardReceiveGate.lean`; censorship pipes |
| W9 | One-command send | VPN / router UX | `its-km send` | `pipe_its_km_pool_e2e.sh` |
| W10 | App egress | SOCKS | `its-pool-proxy` | `pipe_its_socks_pool_e2e.sh` |
| W11 | Addressing | Global directory (e.g. `.i2p`) | PoolMailbox in ciphertext | `--mailbox-fingerprint`; `ParticipationTheorem.lean` |
| W12 | Public infra | Volunteer relays | `deploy/pool-mirror/` | `pipe_its_http_pool_e2e.sh` |
| W13 | Reproducible gates | Release process | E2E pipes + `verify_ecosystem` | `scripts/verify_ecosystem.sh` |

### W1 — numeric example (C under Eve)

Eve owns $10^9$ nodes; Alice sends a 256-bit file. Model claim: $I(M;O)=0$ (`SybilDoctrine`). Integrity floor in Lean Import: $P(\text{forge}) \le 1/p$, $p=2147483647$. Full walkthrough: [MATHEMATICAL_CORE §Va](ITS-routing_MATHEMATICAL_CORE.md).

### W8 — numeric example (ITS-A)

Three mirrors, epochs 0–5. Eve-A omits epoch 3 → `omit_de_whitelists_mirror` removes Eve-A from $\mathcal{M}_{\text{valid}}$. Bob harvests $c_3$ from Eve-B or Charlie; with `witness_pool_urls` and $k=2$, two agreeing witnesses ⇒ `ProofFwd(3,c_3)`. **Outside:** $\mathcal{M}_{\text{valid}}=\emptyset$.

---

## Scope boundaries

| Use ITS when | Use overlay when |
|--------------|------------------|
| File/message to known contact under A0–A3 | General anonymous web browsing, zero setup |
| Shannon C/I claim required in docs | Large existing user base today without operating mirrors |

**A (availability):** **Conditional** — log-proof + whitelist + reroute when valid mirrors/witness exist. Not Shannon “always delivers.”

See: [ITS-routing_PARTICIPATION_SYMMETRY.md](ITS-routing_PARTICIPATION_SYMMETRY.md), [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md), [ITS-routing_OVERLAY_COMPARISON.md](ITS-routing_OVERLAY_COMPARISON.md).
