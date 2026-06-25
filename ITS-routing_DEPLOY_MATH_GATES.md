# Public pool reference deploy — BIS / P1–P3 checklist (D9, M18)

## License: GNU GPLv3 Only

**Reference deploy:** `deploy/pool-mirror/` · **Gate:** `scripts/pipe_its_http_pool_e2e.sh` (M18)

Operators running a public UES pool mirror must satisfy **structural invariants** so Lean theorems under BIS and O⁺ closure (P1–P3) remain honest in production.

---

## Reference deployment

```bash
python3 deploy/pool-mirror/pool_mirror_server.py --port 9191 --store-dir /var/lib/its-pool
```

Behind nginx/CDN in production. Routing clients use:

```toml
[pool]
pool_url = "https://mirror.example.com"
multi_pool_urls = ["https://mirror2.example.com", "https://mirror3.example.com"]
fountain_enabled = true
epoch_interval_ms = 1000
```

**E2E gate:** `ITS_PROD_GATE=1` — no silent file fallback (`pipe_its_http_pool_e2e.sh`).

---

## BIS checklist (Broadcast IP Symmetry)

| ID | Invariant | Deploy requirement | Lean |
|----|-----------|-------------------|------|
| **B1** | Symmetric IP emit per epoch | All benign peers emit same cell rate class; no solo sender spikes | `BroadcastIPSymmetry.lean` — **StructuralAssumption** |
| **B2** | ITS cells indistinguishable from chaff | Fixed `cell_size_L`; pool/AEH stego only — no raw wire in O | **Derived** — `BroadcastIPDerivation.lean` |
| **B3** | Multicast forward without author label | Mirrors store epoch cells only; no src/dst author metadata in HTTP API | `BroadcastForward.lean` |

---

## P1–P3 (O⁺ closure)

| ID | Invariant | Deploy requirement | Lean |
|----|-----------|-------------------|------|
| **P1** | Public pool participation | ≥1 public mirror; operators publish to pool URL(s) | `OplusClosure.lean` |
| **P2** | Constant harvest | Receivers poll all mirrors every epoch (`--continuous`) | `ParticipationSymmetry.lean` L11 |
| **P3** | CoverTransport | Harvest benign E-channels + mirrors; not pool-only idle | `ParticipationSymmetry.lean` L12 |

---

## Pre-ship verification

```bash
./scripts/pipe_its_http_pool_e2e.sh      # M18 — reference mirror
./scripts/pipe_its_pool_e2e.sh           # P8.1
./scripts/pipe_its_censorship_recovery_e2e.sh  # B4 / M21
./scripts/verify_ecosystem.sh /home/user
```

---

## Honest limits

- Mirror operator sees **transcript** (MathSupremacy) — not trusted for C/I.
- Geographic IP remains **operator scope** — mitigated by CoverTransport, not proven away.
- Availability (A) depends on mirror uptime — use `multi_pool_urls` + sneakernet fallback.

See [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md).

---

## Witness operator runbook (examples)

Witnesses are **read-only harvesters** — they publish no user traffic; they attest which epoch cells appeared on mirrors for ValidFwd / `consensusAtEpoch`. Minimum production quorum: **3 witnesses, `consensus_k = 2`**.

### Example: witness systemd unit

```ini
# /etc/systemd/system/its-pool-witness.service
[Unit]
Description=ITS UES pool witness (Charlie)
After=network-online.target

[Service]
Type=simple
User=its
ExecStart=/usr/bin/python3 /opt/its/pool-mirror/pool_mirror_server.py \
  --port 127.0.0.1:8787 --store-dir /var/lib/its-pool-witness/store
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Expose via nginx as `https://witness1.its.example.com` (TLS required for prod). Routing clients add the URL to `witness_pool_urls` only — not `multi_pool_urls`.

### Example: witness harvest smoke

```bash
curl -sf 'https://witness1.its.example.com/pool/cells?from=0' | wc -c
its-routing -c ~/.its/routing.toml client-receive --pool --continuous --timeout-secs 30 \
  --ratchet-seed-file ~/.its/shared-ratchet.seed --out /tmp/witness-smoke.wire
```

### Example: 2-of-3 quorum config

```toml
[pool]
multi_pool_urls = ["https://mirror1.its.example.com", "https://mirror2.its.example.com"]
witness_pool_urls = [
  "https://witness1.its.example.com",
  "https://witness2.its.example.com",
  "https://witness3.its.example.com",
]
consensus_k = 2
valid_fwd_window = 64
```

Two witnesses must agree on the same canonical cell at epoch *e* before a mirror is trusted for forward — see `ValidForwardParty.lean` and `pipe_its_validfwd_e2e.sh`.

Community mirror placeholder list: [deploy/COMMUNITY_MIRRORS.md](deploy/COMMUNITY_MIRRORS.md).

---

## Witness operator runbook (B1–B3, P1–P3)

Witness hosts run the **same** `pool_mirror_server.py` API as mirrors but are listed in `witness_pool_urls` — not `multi_pool_urls`. Operators need **≥3 independent witnesses** and `consensus_k = 2` for 2-of-3 quorum on \(c_e\).

### Pre-flight checklist

| Step | Action | Gate |
|------|--------|------|
| 1 | Deploy mirror API behind nginx + TLS | [deploy/pool-mirror/README.md](deploy/pool-mirror/README.md) § Production |
| 2 | Set `ITS_PROD_GATE=1` on host | No silent file fallback in pipes |
| 3 | Verify HTTPS harvest | `curl -sf 'https://HOST/pool/cells?from=0'` |
| 4 | List origin in `witness_pool_urls` (witness) or `multi_pool_urls` (mirror) | `config.prod.toml` |
| 5 | Confirm B1–B3: symmetric emit, fixed `cell_size_L`, no author metadata | Manual + M18 |
| 6 | Confirm P1–P3: public participation, constant harvest, CoverTransport | M18 + M21 |

### Example witness `routing.toml` snippet

```toml
multi_pool_urls = [
  "https://mirror1.its.example.com",
  "https://mirror2.its.example.com",
]
witness_pool_urls = [
  "https://witness1.its.example.com",  # independent operator
  "https://witness2.its.example.com",
  "https://witness3.its.example.com",
]
consensus_k = 2
valid_fwd_window = 64
fountain_enabled = true
```

### Post-deploy gates

```bash
ITS_PROD_GATE=1 ROUTING/scripts/pipe_its_http_pool_e2e.sh
ROUTING/scripts/pipe_its_censorship_recovery_e2e.sh
ROUTING/scripts/verify_ecosystem.sh /path/to/ecosystem
```

Community fleet registry (placeholder): [deploy/COMMUNITY_MIRRORS.md](deploy/COMMUNITY_MIRRORS.md)

---

## ITS-CHAT gates (M30–M33)

| Gate | Script | Covers |
|------|--------|--------|
| **M30** | `ITS-CHAT/scripts/pipe_its_chat_room_e2e.sh` | `--follow` ≥2 msgs; distinct `room_wire_pk`; unsigned + `--sign --identity`; broadcast `publish_mac`; mute |
| **M31** | `ITS-CHAT/scripts/pipe_its_chat_public_room_e2e.sh` | Registry publish / browse / join (broadcast & chat only) |
| **M32** | `ITS-CHAT/scripts/pipe_its_chat_vote_e2e.sh` | hidden_vote session; pairwise issue/import ballot; cast; tally |
| **M33** | `ITS-CHAT/scripts/pipe_its_chat_archive_e2e.sh` | Frame journal + SSS `.ssc` archive via `sss_chain` |
| **M35** | `ITS-CHAT/scripts/pipe_its_chat_timelock_e2e.sh` | Timelock room seal/unlock (optional ridge) |

Prerequisite: ROUTING `client-receive --follow` (emits `ITS_EPOCH_CURSOR=<n>` after each reconstruct).

```bash
ITS-CHAT/scripts/pipe_its_chat_room_e2e.sh
ITS-CHAT/scripts/pipe_its_chat_public_room_e2e.sh
ITS-CHAT/scripts/pipe_its_chat_vote_e2e.sh
ITS-CHAT/scripts/pipe_its_chat_archive_e2e.sh
```
