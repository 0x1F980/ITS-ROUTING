# ITS dominance pitch — 5-minute script & cards

## License: GNU GPLv3 Only

**Audience:** I2P / Nym-curious operators, mirror hosts, early adopters.  
**Math unchanged:** 0-hop UES pool, Shannon wire, ValidFwd, ReceiveGate — this doc is **narrative only**.

Cross-links: [ITS_OVERLAY_SWITCH.md](../ITS_OVERLAY_SWITCH.md) · [ITS-routing_MATHEMATICAL_CORE.md](../ITS-routing_MATHEMATICAL_CORE.md) §Va · [QUICKSTART.md](../QUICKSTART.md)

---

## 5-minute talk script

**0:00 — Hook**

> "Same threat model you already accept: Eve owns the network, maybe 99.999% of nodes. I2P and Nym bet on *computational* anonymity sets and honest-majority relays. ITS proves **zero bits** of plaintext in observation — Shannon, not lattice hardness."

**0:45 — Sybil (don't fear the node count)**

> "Adding a billion Sybil pool nodes does not add a single confidentiality bit. \(I(M;O)=0\) with or without them. Integrity is bounded at \(1/p\), \(p = 2^{31}-1\). Your verify-oracle runs on **your** endpoint — not Eve's mirror software."

Point to: [CORE §Va](../ITS-routing_MATHEMATICAL_CORE.md) · `SybilDoctrine.lean`

**1:30 — Latency (why it feels faster)**

> "No tunnel build. No mix window. One epoch cell on the UES pool — 0 hops. Tune `epoch_interval_ms`; lab configs hit 50 ms. Compare that to I2P leaseSet + tunnel setup or Nym's mix delay budget."

**2:15 — Availability without volunteer relays**

> "Public pool mirrors + independent witnesses. Eve-A omits epoch 3? De-whitelisted. Harvest from Eve-B or Charlie; k-of-n witnesses agree on \(c_3\) ⇒ ProofFwd. You need **one** mirror in \(\mathcal{M}_{\text{valid}}\) — not a million honest routers."

Gate: `pipe_its_censorship_recovery_e2e.sh` (M21)

**3:00 — Offline killer**

> "I2P dies when the network dies. ITS keeps the **same four commands** — point `--pool-dir` at a USB stick, hand it off, Bob receives. SSS k-of-n tolerates one missing epoch file after copy."

Demo gate: `pipe_its_km_sneakernet_e2e.sh` (M28)

```bash
its-km send --contact bob --file doc.pdf --pool-dir /media/usb/its-pool
# physical handoff
its-km receive --contact alice --out received.pdf --pool-dir /media/usb/its-pool
```

**3:45 — SOCKS & hidden service (pairwise, honest)**

> "Point your app at SOCKS5 localhost — same muscle memory as I2P. Traffic is ITS wire + pool to a **known Bob**, not the whole internet. Hidden-service pattern: Bob runs continuous receive → nginx; Alice uses SOCKS or send. No global `.i2p` directory — pairwise is a **feature** under the Sybil axiom."

Doc: [ITS_HIDDEN_SERVICE.md](../ITS_HIDDEN_SERVICE.md) · Gate: M19

**4:30 — Get started tonight**

> "Constitution path only: vault, contact, prod mirrors, send. One evening migration guide — no raw `client-send`."

Doc: [ITS_MIGRATION_GUIDES.md § Switch in one evening](../ITS_MIGRATION_GUIDES.md#switch-from-i2pnym-in-one-evening)

**4:50 — Close**

> "Overlays win on turnkey browser UX today. ITS wins on math you can verify in Lean and gates you can run locally. Run `./scripts/verify_ecosystem.sh` — green means the story is shipped, not slideware."

---

## Benchmark card (copy-paste for slides / README)

```
┌─────────────────────────────────────────────────────────────────┐
│  LATENCY (typical operator-tuned lab → prod direction)          │
├──────────────────┬──────────────────┬───────────────────────────┤
│  ITS UES pool    │  1 epoch         │  epoch_interval_ms (50–500)│
│  Nym mixnet      │  mix window      │  seconds (layer config)    │
│  I2P             │  tunnel + lease  │  seconds–minutes cold start│
├──────────────────┴──────────────────┴───────────────────────────┤
│  HOPS: ITS 0 · I2P/Nym multi-hop                                │
│  C/I under Eve 99.999%+: ITS unchanged (Lean) · overlays comp.│
│  OFFLINE: ITS USB sneakernet (M28) · I2P requires live net      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Sybil FAQ (bullets)

- **"Won't more nodes help anonymity?"** — Under Eve-majority (A0), overlay k-anonymity **shrinks** when Eve Sybils the set. ITS C/I is **proved unchanged** at 0 extra bits — [§Va](../ITS-routing_MATHEMATICAL_CORE.md), `SybilDoctrine.lean`.
- **"Eve runs all the mirrors."** — She still cannot read wire bytes (C). Forgery capped at \(1/p\) per try (I). If she **omits**, ValidFwd drops her from \(\mathcal{M}_{\text{valid}}\); witnesses + reroute restore A when A2′ holds.
- **"Isn't 0 hops weaker?"** — 0 hops is the **design**: multiset forward + Shannon wire removes path attribution in \(O\) without trusting relay software (`FlowAttributionZero.lean`).
- **"What about global hidden-service directories?"** — Out of scope by design. Pairwise PoolMailbox hints live in ciphertext — no floodfill to Sybil ([ITS_HIDDEN_SERVICE.md](../ITS_HIDDEN_SERVICE.md)).
- **"Can I browse any website?"** — **No** — same honest line as Tor SOCKS to arbitrary clearnet: ITS proxy targets **known Bob** receivers ([ITS-routing_SOCKS_EGRESS.md](../ITS-routing_SOCKS_EGRESS.md)).

---

## Offline killer demo (M28)

**Story:** Alice and Bob have no ISP. Alice writes epoch cells to USB; Eve deletes one file in transit; SSS k-of-n still delivers.

```bash
# After constitution bootstrap (see QUICKSTART)
cp ROUTING/config.offline.toml ~/.its/routing.toml

its-km send --contact bob --file payload.bin --pool-dir /media/usb/its-pool
# unplug, hand USB to Bob
its-km receive --contact alice --out received.bin --pool-dir /media/usb/its-pool
```

**Gate (CI):**

```bash
ROUTING/scripts/pipe_its_km_sneakernet_e2e.sh
```

**Safety gate (prod hazard):** never combine live mirror URLs with `--pool-dir` without offline base — `pipe_its_km_pooldir_prod_hazard.sh` (M28b).

---

## Community mirror list (template)

Document your public fleet in `config.prod.toml` or [deploy/COMMUNITY_MIRRORS.md](../deploy/COMMUNITY_MIRRORS.md):

| Mirror URL | Operator | Witness? | Notes |
|------------|----------|----------|-------|
| `https://mirror1.example/pool` | Team | no | primary harvest |
| `https://mirror2.example/pool` | Community | no | failover |
| `https://witness-charlie.example/pool` | Independent | **yes** | k-of-n quorum |

Deploy reference: [deploy/pool-mirror/README.md](../deploy/pool-mirror/README.md) · Checklist: [ITS-routing_DEPLOY_MATH_GATES.md](../ITS-routing_DEPLOY_MATH_GATES.md)
