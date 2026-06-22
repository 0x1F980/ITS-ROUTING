# ITS hidden service — pairwise Bob site (I2P eepsite analogue)

## License: GNU GPLv3 Only

**Scope:** Known-contact **pairwise** publish/receive — not a global `.i2p`-style directory.  
**Prerequisite:** [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) · pool config · M19 SOCKS gate green locally.

Cross-links: [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md) · [ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md) · [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md)

---

## I2P vs ITS (honest)

| I2P eepsite | ITS pairwise |
|-------------|--------------|
| `.i2p` hostname in global namespace | Contact alias + OOB ratchet sync (`export-qr` / manual seed) |
| Floodfill + tunnel peers | Public UES pool mirrors + optional witnesses |
| SOCKS to destination key | SOCKS5 → ITS wire → pool → Bob `--continuous` receive |
| Directory discoverable (Sybil risk) | Recipient hint **only in ciphertext** — no global registry |

**Why pairwise is a feature:** Under Eve 99.999%+ (A0), a global hidden-service directory is a Sybil surface. ITS keeps addressing local to the pairwise channel — see [CORE §Va](ITS-routing_MATHEMATICAL_CORE.md) and W11 in [ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md).

---

## Architecture

```text
Alice (browser/app)                    Bob (operator)
       │                                      │
       │  SOCKS5 127.0.0.1:1080               │  its-km receive --continuous
       ▼                                      ▼
 its-pool-proxy                         decrypt + forward
       │                                      │
       │  ITS wire cells                      │  local TCP (nginx/backend)
       ▼                                      ▼
   UES pool mirrors  ──────────────────► 127.0.0.1:8080
```

Static publish (no SOCKS): Alice `its-km send --file index.html` → Bob `receive --out site/index.html`.

---

## Bob — receive + nginx

### 1. Constitution receiver (continuous)

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key \
  receive --contact alice --continuous --work-dir /var/its/bob-inbox
```

For SOCKS/streaming egress, pipe decrypted bytes to a local listener (pattern documented in [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md)). For **file-based site publish**, use `--out` per message or a small wrapper that writes incoming files into a web root.

### 2. Ingress bridge (SOCKS / HTTP)

Bob runs a loop (shell or systemd unit):

1. Harvest wire: `its-routing -c routing.toml client-receive --pool --continuous -o req.wire --ratchet-seed-file RATchet`
2. Decrypt: `its_asymmetric decrypt --sk BOB_SK --pk BOB_PK --in req.wire --out req.bin`
3. Forward `req.bin` to local HTTP (e.g. `curl --data-binary @req.bin http://127.0.0.1:8080/` or parse HTTP and proxy)
4. Encrypt response: `its_asymmetric encrypt --pk ALICE_PK --in resp.bin --out resp.wire`
5. Publish: `its-routing client-send --pool -f resp.wire --ratchet-seed-file RATchet`

Reference: `scripts/pipe_its_socks_pool_e2e.sh` (M19 v2 gate).

### 3. Local nginx (static site from received files)

```nginx
server {
    listen 127.0.0.1:8080;
    root /var/its/bob-site;
    index index.html;
}
```

Workflow:

1. Bob receives `index.html` (and assets) via `its-km receive --out /var/its/bob-site/index.html`.
2. nginx serves `127.0.0.1:8080` on Bob's machine only.
3. Alice reaches it **through the pool path** (SOCKS below), not by routing to Bob's raw IP on the public internet.

### 4. Online pool config

```bash
cp ROUTING/config.prod.toml ~/.its/routing.toml
# set multi_pool_urls + witness_pool_urls — see QUICKSTART §2 + deploy/COMMUNITY_MIRRORS.md
```

---

## Alice — SOCKS to Bob's service

### 1. Start pool proxy (production Rust binary)

```bash
cargo build --release -p its_pool_proxy --manifest-path ROUTING/Cargo.toml
its-pool-proxy \
  --listen 127.0.0.1:1080 \
  --config ~/.its/routing.toml \
  --ratchet-seed-file ~/.its/shared-ratchet.seed \
  --pk ~/.its/contacts/bob/public.key \
  --sk ~/.its/keys/alice/secret.key \
  --own-pk ~/.its/keys/alice/public.key
```

> **Deprecated:** `tools/its_pool_proxy.py` — demo only.

### 2. Point app at SOCKS5

```bash
curl --socks5-hostname 127.0.0.1:1080 http://bob-local-service/
# or configure Firefox / app proxy → 127.0.0.1:1080 SOCKS5
```

Bob must run ingress bridge or `receive --continuous` while Alice connects. Gate: `scripts/pipe_its_socks_pool_e2e.sh` (M19 v2).

---

## Simple static publish (no SOCKS)

Alice pushes files directly:

```bash
its-km send --contact bob --file ./site/index.html
its-km send --contact bob --file ./site/style.css
```

Bob:

```bash
its-km receive --contact alice --out /var/its/bob-site/index.html
its-km receive --contact alice --out /var/its/bob-site/style.css
```

Same C/I as messaging — one epoch pool, 0 hops.

---

## PoolMailbox as address

Operators share contacts via vault QR / OOB keys. For pool harvest filtering, use `--mailbox-fingerprint` on receive (hex or fingerprint file). No global hidden-service directory — recipient hint lives in ciphertext only (`RecipientAttributionZero.lean`, W11).

---

## Reference stack (docker-compose)

Bob stack skeleton — pool mirror + nginx + CLI sidecars:

```bash
./deploy/docker/docker_build_all.sh
docker compose -f deploy/docker/docker-compose.bob-stack.yml up -d
curl -sf 'http://127.0.0.1:8787/pool/cells?from=0' >/dev/null && echo pool OK
curl -sf 'http://127.0.0.1:8080/' | head -1
```

See [deploy/docker/docker-compose.bob-stack.yml](deploy/docker/docker-compose.bob-stack.yml) · [deploy/pool-mirror/README.md](deploy/pool-mirror/README.md). Production: nginx + TLS in front of pool mirror API; Bob receiver on secure endpoint.

---

## Verify end-to-end

```bash
ROUTING/scripts/pipe_its_socks_pool_e2e.sh
ROUTING/scripts/pipe_its_hidden_service_e2e.sh   # optional skeleton (delegates to M19)
ROUTING/scripts/verify_ecosystem.sh /home/user
```

---

## Limits (say once)

- Not arbitrary clearnet browsing — **known Bob** only.
- No global `.i2p` directory — contacts are pairwise OOB.
- Full duplex streaming via prod `its-pool-proxy` (no 512-byte clip); constitution file send/receive works today.
