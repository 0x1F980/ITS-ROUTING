# ITS Migration Guides — Replace computational crypto and overlays (Eco F + P8.7)

## License: GNU GPLv3 Only

> **Scope:** This guide covers **messaging and censorship-recovery paths** via `its-km` and pool carrier config. Raw `its_asymmetric encrypt` / `decrypt` remain valid for **non-messaging** static broadcast and file workflows — not as a drop-in replacement for constitution send/receive.

**When ITS wins** vs **when to keep RSA/PQC/Tor/I2P/Nym**.

Product replacement matrix: [ITS-routing_STANDARD_REPLACEMENT.md](ITS-routing_STANDARD_REPLACEMENT.md)  
Overlay comparison (lemma-ID): [ITS-routing_OVERLAY_COMPARISON.md](ITS-routing_OVERLAY_COMPARISON.md)  
Feature → gate one-pager: [ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md)

---

## Switch from I2P/Nym in one evening

**Time budget:** ~2–3 hours for a technical user who already runs I2P or Nym. **Path:** constitution only — no raw `its-routing client-send`.

### Before you start

| I2P / Nym habit | ITS equivalent |
|-----------------|----------------|
| Router / wallet running | `its-km`, `its-routing`, `its_asymmetric` on PATH |
| Destination / Nym ID | Contact alias + OOB transport ratchet sync |
| SOCKS proxy | `its-pool-proxy` on `:1080` (Bob must `--continuous` receive or ingress bridge) |
| `.i2p` hidden site | Pairwise — [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md) |
| Network / floodfill | `multi_pool_urls` + `witness_pool_urls` in `config.prod.toml` |

Read first: [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md) · [ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md)

### Step 1 — Bootstrap (30 min)

```bash
cd /path/to/ecosystem
./ROUTING/scripts/bootstrap.sh
cargo build --release -p its_routing -p its_keymgmt --manifest-path ROUTING/Cargo.toml
cargo build --release --manifest-path ITS-asymmetric/Cargo.toml --bin its_asymmetric --features "bundle,parallel,std,compact-wire"
cargo build --release --manifest-path ITS-KeyManagement/Cargo.toml --bin its-km
export PATH="$PWD/ITS-asymmetric/target/release:$PWD/ROUTING/target/release:$PWD/ITS-KeyManagement/target/release:$PATH"
```

Verify: `ROUTING/scripts/verify_ecosystem.sh /home/user` (or your ecosystem root).

### Step 2 — Vault + contact (20 min)

```bash
its-km vault init --vault-key-dir ~/.its/km-vault-keys
mkdir -p ~/.its
cp ROUTING/config.prod.toml ~/.its/routing.toml
# Edit multi_pool_urls + witness_pool_urls — or run local mirror from deploy/pool-mirror/

its-km --true-secret ~/.its/km-vault-keys/true/secret.key entry add \
  --alias bob --public /path/to/bob.public.key --routing-config ~/.its/routing.toml

its-km export-qr --contact bob --layer transport-ratchet
# Bob imports on his machine:
its-km import-qr --alias alice --layer transport-ratchet --payload 'its-km:qr:...'
```

### Step 3 — First message (10 min)

**Alice:**

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key send --contact bob --file hello.txt
```

**Bob:**

```bash
its-km --true-secret ~/.its/km-vault-keys/true/secret.key receive --contact alice --out received.txt
```

Gate equivalent: `pipe_its_km_pool_e2e.sh` (M27).

### Step 4 — Optional SOCKS (30 min)

If you used I2P SOCKS for apps:

**Bob** (receiver):

```bash
its-km receive --contact alice --continuous
```

**Alice** (proxy):

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

Point app at `SOCKS5 127.0.0.1:1080`. Gate: `pipe_its_socks_pool_e2e.sh` (M19).  
**Not** arbitrary clearnet — known Bob only ([ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md)).

### Step 5 — Optional offline test (15 min)

If you care about air-gap / USB handoff (I2P cannot do this):

```bash
cp ROUTING/config.offline.toml ~/.its/routing.toml
its-km send --contact bob --file doc.pdf --pool-dir /media/usb/its-pool
# hand off USB
its-km receive --contact alice --out doc.pdf --pool-dir /media/usb/its-pool
```

Gate: `pipe_its_km_sneakernet_e2e.sh` (M28). See [QUICKSTART.md](QUICKSTART.md) offline section.

### Step 6 — Hidden service pattern (optional)

Bob nginx + Alice SOCKS or file publish: [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md).

### Done checklist

- [ ] First `send` / `receive` without raw routing CLI  
- [ ] `verify_ecosystem.sh` green on your machine  
- [ ] You can explain Sybil vs mixnet in one sentence ([CORE §Va](ITS-routing_MATHEMATICAL_CORE.md))  
- [ ] You know the honest limit: no arbitrary clearnet browsing  

Operator notes: [ITS-routing_OVERLAY_COMPARISON.md](ITS-routing_OVERLAY_COMPARISON.md)

---

## Use case: Static broadcast on hostile channel

**Replace:** ML-KEM + AES-GCM envelope to published pubkey  
**With:** `its_asymmetric encrypt --pk public.key --in msg --out msg.wire`  
**Why:** Lean K8 Shannon vs computational hardness

---

## Use case: Long-lived messaging

**Replace:** TLS 1.3 static cert only  
**With:** ITS wire + `epoch-advance` + `ITS_WIRE_PROFILE=compact`  
**Why:** Forward secrecy without lattice assumptions

---

## Use case: Tor / I2P / Nym anonymous messaging

**Replace:** Tor hidden service, I2P destination, Nym mixnet client  
**With:** UES Monocell Pool default + `its-km send/receive`  
**Reference:** I(S;O)=0 under Sybil-majority — P6.3 in [ITS-routing_OVERLAY_COMPARISON.md](ITS-routing_OVERLAY_COMPARISON.md)  
**Gate:** `pipe_its_pool_e2e.sh`, `pipe_its_km_pool_e2e.sh`

---

## Use case: Tor SOCKS app egress

**Replace:** Tor Browser SOCKS `127.0.0.1:9050`  
**With:** `its-pool-proxy --listen 127.0.0.1:1080` (see [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md))  
**Why:** Same app integration; Shannon wire + pool instead of onion hops  
**Doc:** [ITS-routing_SOCKS_EGRESS.md](ITS-routing_SOCKS_EGRESS.md) (D30)  
**Gate:** `pipe_its_socks_pool_e2e.sh`

---

## Use case: Public pool infrastructure (operator)

**Replace:** Volunteer Tor relays / I2P floodfill  
**With:** `deploy/pool-mirror/` reference HTTP epoch API  
**Why:** BIS/P1–P3 structural invariants for IP_obs closure  
**Doc:** [ITS-routing_DEPLOY_MATH_GATES.md](ITS-routing_DEPLOY_MATH_GATES.md) (D9)  
**Gate:** `pipe_its_http_pool_e2e.sh` (M18)

---

## Use case: Censored network

**Replace:** Tor bridges, pluggable transports  
**With:** Fountain + `multi_pool_urls` → manual AEH → `its-km send/receive --pool-dir` (sneakernet)  
**Why:** A (availability) recovery without breaking C/I on secure endpoint  
**Doc:** [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md) · [ITS_CONSTITUTION_CLI.md](ITS_CONSTITUTION_CLI.md)  
**Gate:** `pipe_its_censorship_recovery_e2e.sh`, `pipe_its_km_sneakernet_e2e.sh` (M28)

---

## Interop (Eco D)

```bash
# Local proxy + its-curl
python3 ROUTING/tools/its_wire_proxy.py --port 8765 &
./ROUTING/scripts/its-curl.sh http://127.0.0.1:8765/its/wire --pk bob.public.key --file msg.txt
./ROUTING/scripts/pipe_its_proxy_e2e.sh
```

Wire profile: [docs/ITS_WIRE_PROFILE_DRAFT_v0.2.md](docs/ITS_WIRE_PROFILE_DRAFT_v0.2.md)

---

## Use case: Large file + coercion deniability

**Replace:** age + pad (computational)  
**With:** `its_asymmetric encrypt-file` + OOB mapping shares + ITS-timelock  
**Why:** Bundle coercion layer + Shannon wire chunks + C4 deniability  
**Gate:** `pipe_timelock.sh` (M20)

---

## Use case: Signed email / X.509 everywhere

**Keep RSA/Ed25519 OR** use ITS-OTM cert tiers — not drop-in for Chrome/CA yet (Eco D).

---

## Use case: FIPS audit checklist

**Keep NIST algorithms** for compliance label. ITS is mathematical replacement, not certified module (#14).

---

## Use case: IoT / video streaming (bandwidth)

**Consider:** `compact-wire` profile (256² search) or stay on ChaCha — 13n expansion cost on standard profile.

---

## Verify migration

```bash
./scripts/verify_math.sh
./scripts/verify_ecosystem.sh /home/user
```

Cross-links: [ITS-asymmetric_DOMINANCE](../ITS-asymmetric/ITS-asymmetric_DOMINANCE.md) · [ITS_WIRE_PROFILE_DRAFT_v0.1](docs/ITS_WIRE_PROFILE_DRAFT_v0.1.md) · [ITS_OVERLAY_SWITCH.md](ITS_OVERLAY_SWITCH.md) · [ITS_HIDDEN_SERVICE.md](ITS_HIDDEN_SERVICE.md)
