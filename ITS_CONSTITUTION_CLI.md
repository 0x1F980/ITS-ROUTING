# ITS Constitution CLI — operator law

**Read this first.** Everything an operator needs for basic ITS messaging is exactly these seven things. Nothing else is required for send/receive.

Authoritative math: [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) §Va (online ITS-A) and `OfflineChannel` (medium independence).

---

## The seven essentials

| # | What | Command / artifact |
|---|------|-------------------|
| 1 | Vault | `its-km vault init` |
| 2 | Contact + routing profile | `its-km entry add` + `routing.toml` on the entry |
| 3 | Send | `its-km send --contact ALIAS --file PATH` |
| 4 | Receive | `its-km receive --contact ALIAS --out PATH` |
| 5 | Routing config | `routing.toml` — `[pool]` section selects carrier |
| 6 | Binaries on PATH | `its-km`, `its-routing`, `its_asymmetric`, `its-pool-proxy` (SOCKS only) |
| 7 | Key / ratchet OOB | `export-qr` / `import-qr` or manual key + ratchet file |

**QR = identity and transport ratchet only.** Message payload rides the epoch pool (`epoch_*.bin` files or HTTP mirrors) — never QR.

---

## Online vs offline — same four KM commands

The operator surface does not change. Only the `[pool]` section in `routing.toml` (or a one-shot `--pool-dir` override) selects the carrier.

| Profile | Config template | Carrier |
|---------|-----------------|---------|
| **Online** | `config.prod.toml` → `~/.its/routing.toml` | HTTP mirrors (`multi_pool_urls`, `witness_pool_urls`) |
| **Offline / sneakernet** | `config.offline.toml` | Local `pool_file` directory (`epoch_*.bin` on disk/USB) |

```bash
# Online (ITS-A via mirrors + witnesses)
cp ROUTING/config.prod.toml ~/.its/routing.toml
# edit multi_pool_urls, witness_pool_urls

# Air-gap / USB (C/I-first; A via redundant file copies, SSS k-of-n)
cp ROUTING/config.offline.toml ~/.its/routing.toml
# or: its-km entry add --routing-config ROUTING/config.offline.toml ...
```

### Online send/receive

```bash
its-km send --contact bob --file doc.pdf
its-km receive --contact alice --out received.pdf
```

### Offline / removable media

Point the pool at USB (or use `--pool-dir` sugar):

```bash
# Alice writes epoch cells to USB
its-km send --contact bob --file doc.pdf --pool-dir /media/usb/its-pool

# Physically move USB; Bob reads the same pool path
its-km receive --contact alice --out received.pdf --pool-dir /media/usb/its-pool
```

Alternative without the flag: set `pool_file = "/media/usb/its-pool"` in `routing.toml`.

`--pool-dir` writes a temporary `routing.override.toml` in `--work-dir` (copies base config, forces file-only carrier: `pool_file`, clears HTTP mirror URLs, and clears `[aeh].entropy_sources`). Routing logic is unchanged — same `EpochCourier`, different filesystem path.

> **WARNING — unsafe without offline base:** `--pool-dir` **requires** `config.offline.toml` (or equivalent) as the entry's routing base. Using `config.prod.toml` with mirror URLs and then adding `--pool-dir` alone is **UNSAFE**: until the override is applied, prod defaults can still trigger HTTP pool publish and cover-traffic harvest. For air-gap / sneakernet, always set the contact routing config to `ROUTING/config.offline.toml` first, then optionally use `--pool-dir` to point at removable media.

---

## Layer responsibilities

| Layer | Role |
|-------|------|
| **its-km** | Operator; keys, QR, orchestration |
| **its-routing** | Epoch pool, SSS, ValidFwd, ReceiveGate — medium via `EpochCourier` |
| **its_asymmetric** | Wire seal — called by KM |
| **Carrier** | HTTP mirror, `epoch_*.bin` folder, USB copy — config + file I/O only |

Operators should **not** call `its-routing` or `its_asymmetric decrypt` directly in production. Use `its-km send` / `receive`.

### Optional: SOCKS app egress (known Bob / hidden-service)

For I2P-style app egress to a **known peer** (not arbitrary clearnet browsing), use the release proxy — not raw `client-send`:

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

Bob runs `its-km receive --continuous` or an ingress bridge while Alice points her app at `SOCKS5 127.0.0.1:1080`. Gate: `scripts/pipe_its_socks_pool_e2e.sh` (M19 v2). Full doc: [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md) · [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md).

Install via `./scripts/its-operator-install.sh` (includes `its-pool-proxy` on PATH).

---

## What we do not promise the operator

- **Timelock** puzzles (`its-km timelock`, `key wrap-*`) — advanced validity gates, not basic messaging.
- **Dev onion** (`start-node`, mix/UDP features) — regression only, not prod default.
- **Diagnostics** — no `status-audit`; use ecosystem verify scripts instead.
- **ITS-A on total blackout** — witness mirrors need network; offline profile is C/I-first with A from redundant physical copies (SSS k-of-n tolerates one missing `epoch_*.bin`).

---

## Pointers

- Quick path: [QUICKSTART.md](QUICKSTART.md)
- Ecosystem constitution: [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)
- I2P / Nym switch map: [ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md)
- Advanced ridges (not onboarding): [ITS_ADVANCED_RIDGES.md](ITS_ADVANCED_RIDGES.md)
- Censorship ladder: [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md)
- KM manual: [ITS-KeyManagement/ITS-KeyManagement_manual.md](../ITS-KeyManagement/ITS-KeyManagement_manual.md)
