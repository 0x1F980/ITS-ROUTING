# ITS SOCKS egress — Tor SOCKS replacement (D30)

## License: GNU GPLv3 Only

**Gate:** `scripts/pipe_its_socks_pool_e2e.sh` (M19) · **Lemma scope:** L3 constant emit, BIS B2 (derived), CoverTransport L11–L12

---

## What this replaces

| Legacy | ITS equivalent |
|--------|----------------|
| Tor SOCKS5 `127.0.0.1:9050` | **`its-pool-proxy` SOCKS5 `127.0.0.1:1080`** |
| Tor Browser → onion service | App → SOCKS → ITS wire → UES pool → Bob decrypt |
| I2P SOCKS proxy | Same proxy; pool replaces overlay hops |

ITS does **not** provide general anonymous web browsing to arbitrary hosts today — it tunnels app bytes through **Shannon ITS wire + UES pool** to a known contact's receiver. That matches file/message egress and local app proxy use cases under the MathSupremacy threat model.

---

## Quick start

**Bob** (receiver — must be running):

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key receive --contact alice --continuous
```

**Alice** (SOCKS proxy):

```bash
python3 ROUTING/tools/its_pool_proxy.py --listen 127.0.0.1:1080 --config ~/.its/routing.toml
```

Point any SOCKS5-capable app at `127.0.0.1:1080`. Traffic is encrypted with ITS-asymmetric wire, published to the pool, and decrypted only on Bob's math-trusted endpoint.

---

## Operator checklist (BIS / P1–P3)

| Invariant | Operator action |
|-----------|-----------------|
| **P1 — symmetric emit** | Run pool proxy during cover hours; avoid being the only emitter at 03:00 |
| **P2 — constant harvest** | Bob receiver uses `--continuous`; enable CoverTransport mirrors in config |
| **P3 — public pool** | Use `multi_pool_urls` with ≥2 mirrors (`deploy/pool-mirror/`) |
| **B2 — indistinguishable cells** | Default pool path (not raw TCP); AEH only as manual fallback |

See [ITS-routing_DEPLOY_MATH_GATES.md](ITS-routing_DEPLOY_MATH_GATES.md) for full reference-deploy checklist.

---

## Verify

```bash
ROUTING/scripts/pipe_its_socks_pool_e2e.sh
ROUTING/scripts/verify_ecosystem.sh /home/user
```

---

## Related

- [QUICKSTART.md](QUICKSTART.md) — pool send/receive
- [ITS-routing_STANDARD_REPLACEMENT.md](ITS-routing_STANDARD_REPLACEMENT.md) — Tor/I2P/Nym migration
- [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md) — lemma-ID comparison
- Lean: `UnifiedEpochStream.lean`, `BroadcastIPDerivation.lean`, `ParticipationSymmetry.lean`
