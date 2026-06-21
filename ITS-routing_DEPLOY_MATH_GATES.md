# Public pool reference deploy — BIS / P1–P3 checklist (D9, M18)

## License: GNU GPLv3 Only

**Reference deploy:** `deploy/pool-mirror/` · **Gate:** `scripts/pipe_its_http_pool_e2e.sh` (M18)

Operators running a public UES pool mirror must satisfy **structural invariants** so Lean theorems under BIS and O⁺ closure (P1–P3) remain honest in production.

---

## Reference deployment

```bash
python3 deploy/pool-mirror/pool_mirror_server.py --port 9191 --store-dir /var/lib/its-pool
```

Behind nginx/CDN in production. Routing clients use:

```toml
[pool]
pool_url = "https://mirror.example.com"
multi_pool_urls = ["https://mirror2.example.com", "https://mirror3.example.com"]
fountain_enabled = true
epoch_interval_ms = 1000
```

**E2E gate:** `ITS_PROD_GATE=1` — no silent file fallback (`pipe_its_http_pool_e2e.sh`).

---

## BIS checklist (Broadcast IP Symmetry)

| ID | Invariant | Deploy requirement | Lean |
|----|-----------|-------------------|------|
| **B1** | Symmetric IP emit per epoch | All benign peers emit same cell rate class; no solo sender spikes | `BroadcastIPSymmetry.lean` — **StructuralAssumption** |
| **B2** | ITS cells indistinguishable from chaff | Fixed `cell_size_L`; pool/AEH stego only — no raw wire in O | **Derived** — `BroadcastIPDerivation.lean` |
| **B3** | Multicast forward without author label | Mirrors store epoch cells only; no src/dst author metadata in HTTP API | `BroadcastForward.lean` |

---

## P1–P3 (O⁺ closure)

| ID | Invariant | Deploy requirement | Lean |
|----|-----------|-------------------|------|
| **P1** | Public pool participation | ≥1 public mirror; operators publish to pool URL(s) | `OplusClosure.lean` |
| **P2** | Constant harvest | Receivers poll all mirrors every epoch (`--continuous`) | `ParticipationSymmetry.lean` L11 |
| **P3** | CoverTransport | Harvest benign E-channels + mirrors; not pool-only idle | `ParticipationSymmetry.lean` L12 |

---

## Pre-ship verification

```bash
./scripts/pipe_its_http_pool_e2e.sh      # M18 — reference mirror
./scripts/pipe_its_pool_e2e.sh           # P8.1
./scripts/pipe_its_censorship_recovery_e2e.sh  # B4 / M21
./scripts/verify_ecosystem.sh /home/user
```

---

## Honest limits

- Mirror operator sees **transcript** (MathSupremacy) — not trusted for C/I.
- Geographic IP remains **operator scope** — mitigated by CoverTransport, not proven away.
- Availability (A) depends on mirror uptime — use `multi_pool_urls` + sneakernet fallback.

See [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md).
