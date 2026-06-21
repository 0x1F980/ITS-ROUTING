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
| W8 | Censorship | Bridges / many nodes | **Fountain + multi-mirror + AEH** | `scripts/pipe_its_censorship_recovery_e2e.sh`; `scripts/pipe_its_aeh_censorship_e2e.sh` |
| W9 | One-command send | VPN / router start | **`its-km send`** | `scripts/pipe_its_km_pool_e2e.sh`; [QUICKSTART.md](QUICKSTART.md) |
| W10 | App egress | SOCKS 4444 | **`its-pool-proxy :1080`** | `tools/its_pool_proxy.py`; `scripts/pipe_its_socks_pool_e2e.sh` |
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
- **A (availability)** is operational — censorship recovery is documented, not ITS-perfect.

See also: [ITS-routing_PARTICIPATION_SYMMETRY.md](ITS-routing_PARTICIPATION_SYMMETRY.md), [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md), [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md), [ITS-routing_DOMINANCE_PLAN.md](ITS-routing_DOMINANCE_PLAN.md) (master plan v1.6→v2.0, online + sneakernet).
