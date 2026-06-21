# Standard replacement — Tor / I2P / Nym → ITS (B7)

## License: GNU GPLv3 Only

**Gate:** P8.7 · **Docs:** D7 + [ITS_MIGRATION_GUIDES.md](ITS_MIGRATION_GUIDES.md) · **Test:** `verify_ecosystem.sh` M18–M21

---

## Replacement matrix

| Legacy use case | ITS path | Gate / doc |
|-----------------|----------|------------|
| **Tor hidden service → file drop** | UES pool + PoolMailbox (`--mailbox-fingerprint`) | `pipe_its_pool_e2e.sh`; `ParticipationTheorem.lean` |
| **Tor SOCKS browsing** | `its-pool-proxy` SOCKS5 `:1080` | D30 [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md); M19 |
| **I2P `.i2p` destination** | PoolMailbox OTM namespace in ciphertext | B6; `pool_mailbox.rs` |
| **Nym mixnet anonymity** | UES Monocell Pool (0 hops, 1 epoch) | W1–W5 [ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md) |
| **One-command messaging** | `its-km send/receive` | `pipe_its_km_pool_e2e.sh`; M19 |
| **Censorship bridges** | Fountain + multi-mirror + AEH + sneakernet | B4; M21 |
| **Coercion deniability** | ITS-timelock L2 + wire bundle | `pipe_timelock.sh`; M20 |

---

## Side-by-side threat model

| Property | Tor / I2P / Nym | ITS UES |
|----------|-----------------|---------|
| C/I under Eve owns 99.999%+ nodes | Computational (consensus, mix windows) | **Shannon ITS** — `networkEcosystemCertificateV5` |
| Sybil majority | Deanonymization risk | **C/I unchanged** — `SybilDoctrine.lean` |
| Quantum / unbounded compute | PQC migration pressure | **Timeless** — `TimelessSecurity.lean` |
| User mass today | Large existing base | **Roadmap** — honest limit, not math blocker |

Full lemma map: [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md).

---

## Migration steps (operator)

### From Tor Browser SOCKS

1. Bootstrap ecosystem: `./scripts/bootstrap.sh`
2. Configure pool: `config.prod.toml` → `~/.its/routing.toml`
3. Start receiver: `its-km receive --continuous`
4. Start proxy: `python3 tools/its_pool_proxy.py --listen 127.0.0.1:1080`
5. Point SOCKS5 apps at `127.0.0.1:1080`

### From I2P hidden service

1. Exchange ITS public keys + transport ratchet (QR / OOB)
2. Use `--mailbox-fingerprint` instead of `.i2p` hostname
3. Publish via pool default — no onion router daemon

### From Nym

1. Replace mixnet client with `its-km send --contact bob --file …`
2. Accept **0-hop** latency win; read honest A-resilience limits in [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md)

---

## Verify migration readiness

```bash
./scripts/verify_ecosystem.sh /home/user
./scripts/verify_math.sh
```

Both must pass before claiming Tor/I2P/Nym replacement for **file/message/SOCKS egress** under the ITS threat model.
