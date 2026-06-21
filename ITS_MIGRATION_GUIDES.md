# ITS Migration Guides — Replace computational crypto and overlays (Eco F + P8.7)

## License: GNU GPLv3 Only

**When ITS wins** vs **when to keep RSA/PQC/Tor/I2P/Nym**.

Product replacement matrix: [ITS-routing_STANDARD_REPLACEMENT.md](ITS-routing_STANDARD_REPLACEMENT.md)  
Overlay comparison (lemma-ID): [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md)

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
**Why:** I(S;O)=0 under Sybil-majority — see P6.3 in [ITS-routing_OVERLAY_EXTINCTION.md](ITS-routing_OVERLAY_EXTINCTION.md)  
**Gate:** `pipe_its_pool_e2e.sh`, `pipe_its_km_pool_e2e.sh`

---

## Use case: Tor SOCKS app egress

**Replace:** Tor Browser SOCKS `127.0.0.1:9050`  
**With:** `python3 tools/its_pool_proxy.py --listen 127.0.0.1:1080`  
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
**With:** Fountain + `multi_pool_urls` → manual AEH → sneakernet  
**Why:** A (availability) recovery without breaking C/I on secure endpoint  
**Doc:** [ITS-routing_CENSORSHIP_RECOVERY.md](ITS-routing_CENSORSHIP_RECOVERY.md)  
**Gate:** `pipe_its_censorship_recovery_e2e.sh`, `pipe_its_sneakernet_e2e.sh`

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

Cross-links: [ITS-asymmetric_DOMINANCE](../ITS-asymmetric/ITS-asymmetric_DOMINANCE.md) · [ITS_WIRE_PROFILE_DRAFT_v0.1](docs/ITS_WIRE_PROFILE_DRAFT_v0.1.md)
