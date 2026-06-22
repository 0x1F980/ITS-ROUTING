# ITS SOCKS egress — Tor SOCKS replacement (D30)

## License: GNU GPLv3 Only

**Gate:** `scripts/pipe_its_socks_pool_e2e.sh` (M19 v2) · **Lemma scope:** L3 constant emit, BIS B2 (derived), CoverTransport L11–L12

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

**Alice** (SOCKS proxy — production Rust binary):

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

Binary paths default to `its-routing` and `its_asymmetric` on `PATH`, or override via `ITS_ROUTING_BIN` / `ITS_ASYMMETRIC_BIN` (or `--routing` / `--asymmetric`).

Optional sync env (E2E / ingress bridge): `ITS_PROXY_SENT_MARKER` (written after publish), `ITS_PROXY_REPLY_MARKER` (wait before harvest reply).

Point any SOCKS5-capable app at `127.0.0.1:1080`. Traffic is encrypted with ITS-asymmetric wire, published to the pool, and decrypted only on Bob's math-trusted endpoint. Responses stream back **without payload truncation** (full duplex over pool cells).

> **Deprecated:** `tools/its_pool_proxy.py` — demo only; use **`its-pool-proxy`**.

### Constitution / KM path (preferred for operators)

Do **not** call `its-routing client-send` directly in production. For messaging, use `its-km send` / `receive` ([ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md)). SOCKS proxy orchestrates the same stack via subprocess for app egress; long-term, `its-km` may expose SOCKS flags — until then, `its-pool-proxy` is the release binary.

### PoolMailbox contact address

Share a contact via vault QR / `export-qr`, or pass `--mailbox-fingerprint` on `its-routing client-receive` when harvesting for a specific peer. See [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md) (W11) and migration table in [ITS-routing_STANDARD_REPLACEMENT.md](ITS-routing_STANDARD_REPLACEMENT.md).

---

## Bob ingress bridge (hidden-service pattern)

For **pairwise HTTP egress** (I2P eepsite analogue), Bob runs continuous receive and forwards decrypted bytes to a local TCP service:

1. **Local backend** — nginx, `python -m http.server`, or any app on `127.0.0.1:PORT`.
2. **Ingress bridge** — loop: `its-routing client-receive --pool --continuous` → `its_asymmetric decrypt` → forward to local HTTP → encrypt reply with Alice's public key → `client-send --pool`.
3. **Alice** — `its-pool-proxy` or app via SOCKS; pool carries request/response wires.

Full operator guide: [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md).

Example receive (constitution path):

```bash
its-km receive --contact alice --continuous --work-dir /tmp/bob-ingress
# Bridge script forwards plaintext to 127.0.0.1:8080 and publishes replies — see pipe_its_socks_pool_e2e.sh
```

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
ROUTING/scripts/pipe_its_socks_pool_e2e.sh   # M19 v2: Rust proxy + HTTP roundtrip
ROUTING/scripts/verify_ecosystem.sh /home/user
```

---

## Related

- [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md) — Bob ingress + static publish
- [QUICKSTART.md](QUICKSTART.md) — pool send/receive
- [ITS-routing_STANDARD_REPLACEMENT.md](ITS-routing_STANDARD_REPLACEMENT.md) — Tor/I2P/Nym migration
- [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md) — lemma-ID comparison
- Lean: `UnifiedEpochStream.lean`, `BroadcastIPDerivation.lean`, `ParticipationSymmetry.lean`
