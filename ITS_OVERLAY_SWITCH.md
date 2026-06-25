# ITS vs I2P / Nym — operator one-pager

## License: GNU GPLv3 Only

**Read with:** [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) (default path) · [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md) (lemma map) · [docs/ITS_DOMINANCE_PITCH.md](docs/ITS_DOMINANCE_PITCH.md) (5-min pitch)

You are **not** joining a mixnet. You run the constitution CLI (`its-km send/receive`), point `routing.toml` at UES pool mirrors (online) or a USB folder (offline), and get Shannon C/I under Eve 99.999%+ — with **0 hops, 1 epoch** latency instead of multi-hop tunnel setup.

---

## Feature → ITS path → gate

| I2P / Nym expectation | ITS path (constitution) | Gate script |
|------------------------|-------------------------|-------------|
| **One product / wallet** | `its-km vault init` → `entry add` → `send` / `receive` | `pipe_its_km_pool_e2e.sh` (M27) |
| **Network exists** | Public `multi_pool_urls` + `witness_pool_urls` in `config.prod.toml` | `pipe_its_http_pool_e2e.sh` (M18) |
| **Messaging to a contact** | `its-km send --contact ALIAS --file PATH` | `pipe_its_km_pool_e2e.sh` |
| **IRC / rooms (broadcast, chat, vote)** | `its-chat room create` → `send` / `listen --follow` | `ITS-CHAT/scripts/pipe_its_chat_room_e2e.sh` (M30) |
| **Receive loop** | `its-km receive --contact ALIAS --continuous` | `pipe_its_km_pool_e2e.sh` |
| **SOCKS app egress** | `its-pool-proxy --listen 127.0.0.1:1080` + Bob `--continuous` / ingress bridge | `pipe_its_socks_pool_e2e.sh` (M19 v2) |
| **Hidden service / site** | Bob: `receive --continuous` → local nginx; Alice: SOCKS or `send --file` | [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md) · M19 |
| **Contact address** | Vault alias + OOB ratchet sync (`export-qr` / `import-qr`); **PoolMailbox** `--mailbox-fingerprint` on receive | `--mailbox-fingerprint` · W11 |
| **Offline / air-gap** | `config.offline.toml` or `--pool-dir /media/usb/its-pool` | `pipe_its_km_sneakernet_e2e.sh` (M28) |
| **Censorship / bridges** | Fountain + mirror failover → AEH → sneakernet | `pipe_its_censorship_recovery_e2e.sh` (M21) |
| **Sybil-majority threat** | C/I unchanged at 0 bits; ValidFwd de-whitelists omitters | `verify_math.sh` · CORE §Va |
| **Coercion deniability** | `its-km timelock` (advanced ridge) | `pipe_timelock.sh` (M20) |
| **Full ecosystem ship** | Bootstrap + verify | `verify_ecosystem.sh` (M17–M22) |

---

## Why switch (30 seconds)

| Dimension | I2P / Nym intuition | ITS answer |
|-----------|---------------------|------------|
| **Latency** | Tunnel build + mix window (seconds–minutes) | **1 epoch** — `epoch_interval_ms` in prod config (often 50–500 ms lab, operator-tuned live) |
| **Sybil** | More honest nodes ⇒ bigger anonymity set | **C/I unchanged** if Eve owns 99.999%+ nodes — see [CORE §Va](ITS-routing_MATHEMATICAL_CORE.md) |
| **Offline** | Router dies without network | **USB sneakernet** — same four KM commands, `config.offline.toml` |
| **Hops** | k-anonymity via relays | **0 hops** UES pool — multiset forward, Shannon wire |

Full positive pitch + honest boundaries: [ITS-routing_OVERLAY_EXTINCTION.md § Why ITS over I2P/Nym](ITS-routing_OVERLAY_EXTINCTION.md#why-its-over-i2pnym).

---

## Honest scope (one line)

ITS SOCKS and hidden-service patterns target **known Bob** — not arbitrary clearnet browsing like Tor Browser to any host.

---

## Migration paths

| From | Doc |
|------|-----|
| I2P / Nym evening switch | [ITS_MIGRATION_GUIDES.md § Switch in one evening](ITS_MIGRATION_GUIDES.md#switch-from-i2pnym-in-one-evening) |
| Tor SOCKS | [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md) |
| Lemma-by-lemma map | [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md) |

---

## Verify

```bash
./scripts/verify_ecosystem.sh /home/user
```
