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

Docker smoke (prebuilds v2):

```bash
./deploy/docker/docker_build_all.sh
docker compose -f deploy/docker/docker-compose.yml up -d pool-mirror
curl -sf 'http://127.0.0.1:8787/pool/cells?from=0' >/dev/null && echo OK
```

---

## Production playbook — nginx + TLS + ITS_PROD_GATE

Public mirrors must satisfy B1–B3 / P1–P3 (see [ITS-routing_DEPLOY_MATH_GATES.md](../../ITS-routing_DEPLOY_MATH_GATES.md)). This section is the operator runbook for a **production** mirror host.

### 1. Host layout

```text
/var/lib/its-pool-mirror/store/   # epoch_*.bin cells (no metadata)
/etc/nginx/sites-available/its-pool-mirror
```

Run the reference server behind nginx (systemd or container):

```bash
python3 pool_mirror_server.py --port 127.0.0.1:8787 --store-dir /var/lib/its-pool-mirror/store
```

Or use the Docker image from `deploy/docker/docker-compose.yml` with `ITS_PROD_GATE=1`.

### 2. nginx reverse proxy (TLS)

Example site config — replace `mirror1.its.example.com` with your hostname:

```nginx
upstream its_pool_mirror {
    server 127.0.0.1:8787;
    keepalive 8;
}

server {
    listen 443 ssl http2;
    server_name mirror1.its.example.com;   # REPLACE

    ssl_certificate     /etc/letsencrypt/live/mirror1.its.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mirror1.its.example.com/privkey.pem;

    # Pool API only — no directory listing, no extra paths
    location /pool/ {
        proxy_pass http://its_pool_mirror;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        client_max_body_size 16m;
    }

    location / {
        return 404;
    }
}
```

Obtain certificates (Let's Encrypt or your PKI), enable the site, reload nginx.

List the **HTTPS origin** (no path suffix) in operator `routing.toml`:

```toml
multi_pool_urls = [
  "https://mirror1.its.example.com",   # REPLACE
  "https://mirror2.its.example.com",   # REPLACE
]
```

### 3. ITS_PROD_GATE=1

Set `ITS_PROD_GATE=1` on the mirror host and in CI/pipe environments when verifying against **public** URLs. This disables silent file-fallback behaviour that is acceptable in lab pipes only.

```bash
export ITS_PROD_GATE=1
docker compose -f deploy/docker/docker-compose.yml up -d pool-mirror
```

Gate scripts (`pipe_its_http_pool_e2e.sh`, ecosystem verify against public mirrors) expect real HTTP harvest — not localhost file pools.

### 4. Witness mirrors (P1–P3)

Witness hosts use the **same** `pool_mirror_server.py` API but are listed separately:

```toml
witness_pool_urls = [
  "https://witness1.its.example.com",  # REPLACE — independent operator
  "https://witness2.its.example.com",
  "https://witness3.its.example.com",
]
consensus_k = 2
```

Requirements:

| ID | Requirement |
|----|-------------|
| B1 | Symmetric cell rate across peers (no solo off-hours sender) |
| B2 | Fixed `cell_size_L`; cells are 𝒟_IP draws |
| B3 | Store epoch cells only — no author/src metadata |
| P1 | Each mirror listed in `multi_pool_urls` |
| P2 | Receivers poll all mirrors every epoch |
| P3 | CoverTransport harvests benign channels, not pool-only idle |

**Gate:** `scripts/pipe_its_http_pool_e2e.sh` · Full checklist: [ITS-routing_DEPLOY_MATH_GATES.md](../../ITS-routing_DEPLOY_MATH_GATES.md)

### 5. Post-deploy verification

```bash
# From operator workstation (HTTPS mirror URL configured in routing.toml)
curl -sf 'https://mirror1.its.example.com/pool/cells?from=0' >/dev/null

ITS_PROD_GATE=1 ROUTING/scripts/pipe_its_http_pool_e2e.sh
ROUTING/scripts/verify_ecosystem.sh /path/to/ecosystem
```

Document your public mirror URLs in `config.prod.toml` (REPLACE placeholders) before shipping default prod config to operators.

Community fleet registry: [COMMUNITY_MIRRORS.md](../COMMUNITY_MIRRORS.md)

---

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
