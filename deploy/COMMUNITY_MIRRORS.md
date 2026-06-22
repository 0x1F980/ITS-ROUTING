# Community public mirror fleet (placeholder)

## License: GNU GPLv3 Only

**Status:** Template for early-adopter operators. Replace placeholder hostnames before production use.

ITS public availability (ITS-A) requires **≥2 independent UES pool mirrors** and **≥3 witness mirrors** with `consensus_k = 2` in [`config.prod.toml`](../config.prod.toml). This file lists community-operated endpoints once they are live — not maintained by core devs until sign-off.

---

## How to list your mirror

1. Deploy per [`deploy/pool-mirror/README.md`](pool-mirror/README.md) (nginx + TLS + `ITS_PROD_GATE=1`).
2. Run witness checklist in [`ITS-routing_DEPLOY_MATH_GATES.md`](../ITS-routing_DEPLOY_MATH_GATES.md) B1–B3, P1–P3.
3. Open a PR adding your row below (HTTPS URL, operator contact, jurisdiction note).

---

## Public UES pool mirrors (replace before ship)

| Mirror URL | Operator | Region | Status |
|------------|----------|--------|--------|
| `https://mirror1.its.example.com` | **REPLACE** | — | placeholder |
| `https://mirror2.its.example.com` | **REPLACE** | — | placeholder |
| `https://mirror3.its.example.com` | **REPLACE** | — | optional third |

Paste into `routing.toml`:

```toml
multi_pool_urls = [
  "https://mirror1.its.example.com",
  "https://mirror2.its.example.com",
]
```

---

## Witness fleet (independent of mirror operator)

| Witness URL | Operator | Notes |
|-------------|----------|-------|
| `https://witness1.its.example.com` | **REPLACE** | must not share infra with mirror1 |
| `https://witness2.its.example.com` | **REPLACE** | Charlie-style read-only harvest |
| `https://witness3.its.example.com` | **REPLACE** | 2-of-3 quorum with `consensus_k = 2` |

```toml
witness_pool_urls = [
  "https://witness1.its.example.com",
  "https://witness2.its.example.com",
  "https://witness3.its.example.com",
]
consensus_k = 2
valid_fwd_window = 64
```

---

## Verify against public URLs

After URLs are live (not placeholders):

```bash
ITS_PROD_GATE=1 ROUTING/scripts/pipe_its_http_pool_e2e.sh
ROUTING/scripts/verify_ecosystem.sh /path/to/ecosystem
```

Gate: M18 HTTP pool + M21 censorship recovery green against **public** mirror URLs — see [ITS_OVERLAY_SWITCH.md](../ITS_OVERLAY_SWITCH.md).

---

## Related

- [config.prod.toml](../config.prod.toml) — default onboarding template
- [ITS-routing_CENSORSHIP_RECOVERY.md](../ITS-routing_CENSORSHIP_RECOVERY.md) — fountain failover
- [docs/ITS_DOMINANCE_PITCH.md](../docs/ITS_DOMINANCE_PITCH.md) — community mirror ask (Fase 6)
