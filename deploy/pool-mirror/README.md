# UES Pool Mirror — reference deploy

Minimal HTTP epoch API for UES Monocell Pool:

- `POST /pool/cell?epoch=N` — publish fixed-size cell body
- `GET /pool/cells?from=N` — harvest cells (binary: `epoch:u64|len:u32|bytes`)

## Local test

E2E pipes pick a **free local port** automatically. For manual testing:

```bash
python3 deploy/pool-mirror/pool_mirror_server.py --port 9191 --store-dir /tmp/pool-mirror
```

Set routing config:

```toml
[pool]
pool_url = "http://127.0.0.1:9191"
multi_pool_urls = []
```

Or use `multi_pool_urls` for A-resilience across mirrors.

## Production

Deploy behind nginx or CDN static origin. Use `ITS_PROD_GATE=1` when verifying — no silent file fallback.

## BIS / P1–P3 checklist (M18)

Public mirrors must satisfy structural invariants for IP_obs closure:

| ID | Requirement |
|----|-------------|
| B1 | Peers emit symmetric cell rate (no solo 03:00 sender) |
| B2 | Fixed `cell_size_L`; cells are 𝒟_IP draws (derived in Lean) |
| B3 | API stores epoch cells only — no author/src metadata |
| P1 | Listed in `multi_pool_urls` for redundancy |
| P2 | Receivers poll all mirrors every epoch |
| P3 | CoverTransport harvests benign channels, not pool-only idle |

Full checklist: [ITS-routing_DEPLOY_MATH_GATES.md](../../ITS-routing_DEPLOY_MATH_GATES.md)  
**Gate:** `scripts/pipe_its_http_pool_e2e.sh`
