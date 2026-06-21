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
