# ITS-routing: What stays in this repo (transport boundary)

## License: GNU GPLv3 Only

**Operator identity** (contacts, vault, passwords, duress policy) lives in **[ITS-KeyManagement](https://github.com/0x1F980/ITS-KeyManagement)**. This document lists what **remains** in ITS-routing after the identity migration.

---

## In scope (transport daemon)

| Component | Location | Role |
|-----------|----------|------|
| Onion routing, UDP I/O | `its_routing/src/main.rs` | Bytes on wire |
| `routing_table`, node `id`, port | `config.toml` | Hop topology (not human contacts) |
| Constant-rate chaff, Lorenz jitter | `start-node`, traffic config | Traffic shaping |
| SSS fragment / reconstruct | `client-send`, `client-receive` | Message splitting over network |
| `StealthIdentity` | AEH stego paths | Crypto stego (`core_logic`) — **not** a contact book |
| Expert pipes | `time-lock`, `time-unlock`, `time-deny`, `fingerprint-erasure` | Low-level CLI for power users |
| Analog share export/import | `client-export-share`, `client-import-share` | Transport-adjacent SSS strings |
| `--ratchet-seed-file` | AEH send/receive | **32-byte seed from ITS-KeyManagement** — routing never sees passwords |

---

## Out of scope (moved to ITS-KeyManagement)

| Removed from routing | Now in ITS-KeyManagement |
|----------------------|---------------|
| `--password` / `--duress` on send/receive | `its-km vault unlock`, `export-ratchet-seed` |
| PBKDF2 dual salts | `its_id::ratchet` constants |
| `client-vault-unlock` (was doc-only) | `its-km vault` |
| Contact aliases in usecase examples | `its-km contact` |

---

## Non-production demo fallback

If `client-send` / `client-receive` run **without** `--ratchet-seed-file`, AEH mode uses anchor + whitening from `[crypto]` in `config.toml`. This is **lab/demo only**. Production flows must use:

```bash
its-km export-ratchet-seed --out /tmp/seed.bin --password '...' [--duress]
its-routing client-send -c config.toml -f payload.bin -d 2 --aeh --ratchet-seed-file /tmp/seed.bin
```

See [ITS-KeyManagement_PIPE.md](https://github.com/0x1F980/ITS-KeyManagement/blob/main/ITS-KeyManagement_PIPE.md).
