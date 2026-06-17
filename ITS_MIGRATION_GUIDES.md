# ITS Migration Guides — Replace computational crypto (Eco F)

## License: GNU GPLv3 Only

**When ITS wins** vs **when to keep RSA/PQC**.

---

## Use case: Static broadcast on hostile channel

**Replace:** ML-KEM + AES-GCM envelope to published pubkey  
**With:** `its_asymmetric encrypt --pk public.key --in msg --out msg.wire`  
**Why:** Lean K8 Shannon vs computational hardness

---

## Use case: Long-lived messaging

**Replace:** TLS 1.3 static cert only  
**With:** ITS wire + epoch-advance + (future) ITS-session ratchet  
**Why:** Forward secrecy without lattice assumptions

---

## Use case: Large file + coercion deniability

**Replace:** age + pad (computational)  
**With:** `its_asymmetric encrypt-file` + OOB mapping shares  
**Why:** Bundle coercion layer + Shannon wire chunks

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

Cross-links: [ITS-asymmetric_DOMINANCE](../ITS-asymmetric/ITS-asymmetric_DOMINANCE.md) · [ITS_WIRE_PROFILE_DRAFT_v0.1](docs/ITS_WIRE_PROFILE_DRAFT_v0.1.md)
