# ITS Ecosystem — Constitution (single entry)

## License: GNU GPLv3 Only

**Read this document first.** Execution plan: `.cursor/plans/its_klippe_v4_fundament.plan.md` (do not fork without updating gates).

---

## ITS-first charter

1. **Wire confidentiality** = Shannon ITS — Lean in [ITS-asymmetric](../ITS-asymmetric/ITS-asymmetric_FORMAL_VERIFICATION.md).
2. **SSS chaining** = single algebra in [SSS_CHAIN](../SSS_CHAIN/SSS_CHAIN_SECURITY_LAYERS.md).
3. **No computational fallback** as equivalent wire crypto in math repos.
4. **PQC/RSA** = out of math-klippe scope (centuries horizon, not four-decade hardness).
5. **Performance** (`compact-wire`) = faster ITS, not weaker ITS.
6. **One truth per concern** — no hidden monolith, no duplicate field math, one wire-FS story.
7. **Freeze after v1.0.0** — math + transport change only on security fixes.

---

## Three layers

| Layer | Repos | Security type |
|-------|-------|---------------|
| **Math-klippe** (frozen v1.0.0) | SSS_CHAIN, ITS-asymmetric, ITS-OTM, ITS-timelock | Information-theoretic (per layer docs) |
| **Transport** | ROUTING (`its_transport`, `its_routing`) | Opaque bytes + transport ratchet |
| **Glue** | ITS-KeyManagement | Subprocess orchestration only |
| **Operational ridges** | ITS-hardware, ITS-fingerprint_erasure, ITS-ledger | Honest scope in each `*_SECURITY_LAYERS.md` |

---

## Repo register

| Repo | Crate | Owns | Must NOT own |
|------|-------|------|--------------|
| [SSS_CHAIN](../SSS_CHAIN) | `sss_chain` | Field, Lagrange, link, epoch, OTM helpers | Wire encrypt, transport |
| [ITS-asymmetric](../ITS-asymmetric) | `its_asymmetric` | Shannon wire, bundle, epoch FS | Onion, contacts |
| [ITS-OTM](../ITS-OTM_public_attestation) | `its_otm_public_attestation` | WC-MAC attestation | Wire OTP body |
| [ITS-timelock](../ITS-self_enclosed_timelock) | `its_self_enclosed_timelock` | RSW delay + SSS ITS L2 | Transport |
| [ROUTING/its_transport](its_transport) | `its_transport` | Onion, fragment, transport ratchet, tunnel | Shannon wire |
| [ROUTING/its_routing](its_routing) | `its_routing` | Daemon, UDP, chaff, pipes | Crypto proofs, vault |
| [ITS-KeyManagement](../ITS-KeyManagement) | `its_keymgmt` | Vault, contacts, send/receive glue | Shannon implementation |
| [ITS-hardware](../ITS-hardware) | `its_hardware` | TRNG, seL4, Lorenz HAL, analog shares | Wire proofs |
| [ITS-ledger](../ITS-ledger) | `its_ledger` | Endpoint vault, AEH hash fetch | Operator keyring |
| [ITS-fingerprint_erasure](../ITS-fingerprint_erasure) | `its_fingerprint_erasure` | Γ normalization | Wire ITS |

**Rules:** ROUTING has **no** Cargo dep on `its_asymmetric`. KeyManagement has **no** Cargo dep on sibling ITS crates.

---

## Wire forward secrecy (v1)

| Mechanism | Role |
|-----------|------|
| `its_asymmetric epoch-advance` | **Canonical** wire FS (SSS_CHAIN) |
| `its_transport::StateRatchet` | Transport/onion/AEH only |
| ~~`its_sessions`~~ | **Removed** — was computational overlap |

---

## Proof manifests

| Repo | Document |
|------|----------|
| ITS-asymmetric | [PROOF_MANIFEST.md](../ITS-asymmetric/PROOF_MANIFEST.md) |
| SSS_CHAIN | [PROOF_MANIFEST.md](../SSS_CHAIN/PROOF_MANIFEST.md) |
| ITS-OTM | [PROOF_MANIFEST.md](../ITS-OTM_public_attestation/PROOF_MANIFEST.md) |
| ITS-timelock | [PROOF_MANIFEST.md](../ITS-self_enclosed_timelock/PROOF_MANIFEST.md) |

---

## Lorenz ownership

**Implementation:** `its_transport::lorenz` (onion mixing-node delay). **HAL API:** ITS-hardware re-exports the same module for TRNG/chaff timing tests. No duplicate `lorenz.rs` in hardware.

---

## Legacy (archived)

| Legacy | Successor |
|--------|-----------|
| `core_logic` / ITS-session repo | `its_transport` + ITS-asymmetric |
| `ITS-net` / `morphic-its` | `its-routing` |
| `hydra_sss` | `sss_fragment` |
| `ITS-session` wire FS | asymmetric `epoch-advance` |

Archive tag: `core_logic-archive-v1` on [ITS-session](../ITS-session).

---

## Build & verify

```bash
./scripts/bootstrap.sh      # clone 0x1F980 ecosystem (tags)
./scripts/verify_ecosystem.sh
```

---

## Organization

Active development: **0x1F980** on GitHub. Legacy **0x1F464** references are forbidden in `Cargo.toml` after v1.0.0.

---

## Out of v1 math scope

PQC/RSA wire alternative · ITS-garbled/MPC · Web PKI/TLS · FIPS module as math replacement · monorepo · computational `its_sessions`
