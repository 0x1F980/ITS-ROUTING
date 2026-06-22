# Hidden service reference stack (Fase 4)

## License: GNU GPLv3 Only

Pairwise Bob site via UES pool — **not** a global `.i2p` directory. See [ITS_HIDDEN_SERVICE.md](../../ITS_HIDDEN_SERVICE.md).

---

## Local compose profile

The `hidden-service` profile adds Bob-side nginx + a receive sidecar pattern on top of the pool mirror stack:

```bash
./deploy/docker/docker_build_all.sh
docker compose -f deploy/docker/docker-compose.yml --profile hidden-service up -d
```

Services:

| Service | Role |
|---------|------|
| `pool-mirror` | UES epoch API (always-on) |
| `bob-nginx` | Static site on `127.0.0.1:8080` inside compose network |
| `its-routing` | Exec target for ingress bridge scripts |
| `its-km` | Constitution receive (`receive --continuous`) |

---

## Bob ingress (manual bridge)

Inside the routing container or host PATH:

1. `its-km receive --contact alice --continuous --work-dir /var/its/inbox`
2. Loop: decrypt → forward to `bob-nginx:80` → encrypt reply → `client-send --pool`

Reference gate: `scripts/pipe_its_socks_pool_e2e.sh` (M19 v2) and skeleton `scripts/pipe_its_hidden_service_e2e.sh`.

---

## Alice access

- **SOCKS:** `its-pool-proxy --listen 127.0.0.1:1080` — [ITS-routing_SOCKS_EGRESS.md](../../ITS-routing_SOCKS_EGRESS.md)
- **Files:** `its-km send --contact bob --file index.html`

---

## Verify

```bash
ROUTING/scripts/pipe_its_hidden_service_e2e.sh   # skeleton — extends M19
ROUTING/scripts/pipe_its_socks_pool_e2e.sh        # full HTTP roundtrip gate
```
