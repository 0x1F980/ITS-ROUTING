# ITS-routing: Superiority vs I2P / Nym

## Win-conditions (W1–W13)

| # | Criterion | I2P / Nym | ITS UES v2.0 | Verify |
|---|-----------|-----------|--------------|--------|
| W1 | C/I under active Eve | Computational | **ITS** I(S;O)=0 | Lean `UnifiedEpochStream.lean`; `scripts/pipe_its_pool_e2e.sh` |
| W2 | Sybil 98% nodes | Deanonymization risk | **C/I unchanged** | Lean `SybilDoctrine.lean`; pool pipe |
| W3 | N=1 user | Needs overlay mass | **FewUserDoctrine** | Lean `FewUserDoctrine.lean` |
| W4 | Quantum / unbounded compute | PQC migration | **MathSupremacy** | Lean `MathSupremacyDoctrine.lean` |
| W5 | Latency | Multi-hop + mix window | **0 hops, 1 epoch** | `epoch_interval_ms` in `config.prod.toml` |
| W6 | Idle leak | Possible | **L3 constant emit** | Epoch-loop in `client.rs`; cover pipe |
| W7 | O⁺ participation | Hops hide pattern | **CoverTransport** | `scripts/pipe_its_cover_harvest_e2e.sh`; L11–L12 Lean |
| W8 | Censorship | Bridges / many nodes | **ITS-A (v9): ProofFwd + \(\mathcal{M}_{\text{valid}}\) + witness k-of-n + SSS** | Lean `ForwardProof.lean`, `ValidForwardParty.lean`, `WitnessConsensus.lean`, `ForwardReceiveGate.lean`; `scripts/pipe_its_censorship_recovery_e2e.sh`; `scripts/pipe_its_aeh_censorship_e2e.sh` |

### W1 — numeric example (C + I under Eve 99.999%+)

Eve owns \(10^9\) nodes; Alice sends a 256-bit file. Channel observation \(O\) reveals **0 bits** about \(M\): \(I(M;O)=0\) (`SybilDoctrine` — adding \(10^9\) Sybil posters changes nothing). Integrity: each forged cell accepted with probability \(\leq 1/p\), \(p=2147483647\). A \(10^6\)-cell transfer expects \(\leq 10^6/p \approx 0.0005\) false accepts — OTM verify on Bob's A2′ endpoint only. Full tables: [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) §Va.

### W8 — numeric example (ITS-A vs censorship)

Three mirrors, epochs 0–5. Eve-A selectively omits epoch 3 → `omit_de_whitelists_mirror` removes Eve-A from \(\mathcal{M}_{\text{valid}}\). Bob harvests \(c_3\) from Eve-B or Charlie; with `witness_pool_urls` and \(k=2, n=3\), two witnesses agreeing on \(c_3\) ⇒ `ProofFwd(3,c_3)`. Against \(10^9\) Eve nodes, **one** valid forwarder in \(\mathcal{M}_{\text{valid}}\) is enough. Outside: \(\mathcal{M}_{\text{valid}}=\emptyset\). Walkthrough: [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) § Eve scenario · §Va in CORE.

| W9 | One-command send | VPN / router start | **`its-km send`** | `scripts/pipe_its_km_pool_e2e.sh`; [QUICKSTART.md](QUICKSTART.md) |
| W10 | App egress | SOCKS 4444 | **`its-pool-proxy :1080`** | `its_pool_proxy/` crate; `scripts/pipe_its_socks_pool_e2e.sh` |
| W11 | Hidden addressing | `.i2p` | **PoolMailbox** (OTM in ciphertext) | `--mailbox-fingerprint`; Lean `ParticipationTheorem.lean` |
| W12 | Public infrastructure | Volunteer relays | **`deploy/pool-mirror/`** | `scripts/pipe_its_http_pool_e2e.sh`; `multi_pool_urls` |
| W13 | Reproducible ship | Releases | **9 E2E pipes + verify_ecosystem** | `scripts/verify_ecosystem.sh`; tag `v2.0.0` |

Run all gates:

```bash
./ROUTING/scripts/verify_ecosystem.sh
```

## When to choose ITS

- You need **information-theoretic** C/I under an adversary who owns the network.
- You send **files/messages** to known contacts (not general anonymous browsing as primary use).
- You accept **secure endpoint** discipline (local keys, OTM verify).

## When I2P/Nym still fit

- Primary need is **anonymous web browsing** with zero setup.
- You want a **large existing user base** today without operating pool mirrors.

## Honest limits

- Raw IP geography remains **operator/axiom** scope — mitigated by CoverTransport (L11–L12), not proven away.
- **A (availability)** is **100% ITS-math (v9)**: log-proof + whitelist + reroute when valid mirrors/witness exist — not Shannon “always delivers.” **Outside:** total blackout without A2′ witness + empty \(\mathcal{M}_{\text{valid}}\); otherwise ITS-A reroute via ProofFwd / ReceiveGate.

See also: [ITS-routing_PARTICIPATION_SYMMETRY.md](ITS-routing_PARTICIPATION_SYMMETRY.md), [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md), [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md), [ITS-routing_DOMINANCE_PLAN.md](ITS-routing_DOMINANCE_PLAN.md) (master plan v1.6→v2.0, online + sneakernet).
