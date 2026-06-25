# ITS-routing: Security Layers & ITS Scope (ITS-routing_SECURITY_LAYERS)

## License: GNU GPLv3 Only
## Target: Auditors, AI-assisted reviewers, integrators

**Read this document first** before auditing this CLI/daemon repository.

**Ecosystem master:** [ITS_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-ROUTING/blob/master/ITS_ECOSYSTEM.md) (ITS core repo).

---

## 1. Role

ITS-routing **orchestrates** upstream crates. It does not replace their proofs — it routes operators to the correct **per-layer** scope document.

---

## 2. Subcommand → upstream ITS layers

| Subcommand / flag | Upstream crate | ITS scope doc |
|-------------------|----------------|---------------|
| ITS-asymmetric wire / bundle | ITS-asymmetric | [ITS-asymmetric_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-asymmetric/blob/main/ITS-asymmetric_SECURITY_LAYERS.md), [UNIFIED_THREAT_MODEL](https://github.com/0x1F980/ITS-asymmetric/blob/main/ITS-asymmetric_UNIFIED_THREAT_MODEL.md), [Wire Profile draft](docs/ITS_WIRE_PROFILE_DRAFT_v0.1.md) |
| Operator vault, contacts, `--ratchet-seed-file` source | **ITS-KeyManagement** | [ITS-KeyManagement_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-KeyManagement/blob/main/ITS-KeyManagement_SECURITY_LAYERS.md) |
| `--ratchet-seed-file` (transport) | ITS-routing AEH path | This repo — accepts **derived bytes only**, never passwords |
| Default send stack (Γ + OTP + chaff) | ITS + ITS-fingerprint_erasure | [ITS_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-ROUTING/blob/master/ITS_ECOSYSTEM.md), [ITS-fingerprint_erasure_ITS_SCOPE.md](https://github.com/0x1F980/ITS-FINGERPRINT_ERASURE/blob/master/ITS-fingerprint_erasure_ITS_SCOPE.md) |
| `--strict-stack` | FE + ITS wire stack | FE scope + core ITS §3 |
| `time-lock` / `time-unlock` / `time-deny` | ITS-self_enclosed_timelock | [ITS-self_enclosed_timelock_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-self_enclosed_timelock/blob/master/ITS-self_enclosed_timelock_SECURITY_LAYERS.md) |
| OTM / AEH attestation verify | ITS-OTM_public_attestation | [ITS-OTM_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-OTM_public_attestation/blob/main/ITS-OTM_SECURITY_LAYERS.md) |
| `--fingerprint-erasure` | ITS-fingerprint_erasure | [ITS-fingerprint_erasure_ITS_SCOPE.md](https://github.com/0x1F980/ITS-FINGERPRINT_ERASURE/blob/master/ITS-fingerprint_erasure_ITS_SCOPE.md) |
| Hardware entropy hooks | sidechannel_resistant_hardware | [ITS-hardware_SECURITY_LAYERS.md](https://github.com/0x1F980/sidechannel_resistant_hardware/blob/master/ITS-hardware_SECURITY_LAYERS.md) |
| Ledger / vault sync | ITS-ledger | [ITS-ledger_SECURITY_LAYERS.md](https://github.com/0x1F980/ITS-ledger/blob/master/ITS-ledger_SECURITY_LAYERS.md) |

---

## 3. Default stack (documentation)

When operators use the full offline send path without disabling layers:

1. **Γ** — fingerprint erasure (discrete ITS subset + axiomatic continuous rules)
2. **OTP** — $C=M+K$ over $\mathbb{F}_p$ (ITS given uniform keys)
3. **Chaff** — ITS-indistinguishable dummy packets (`create_chaff_onion_packet` ≡ real distribution; Lean `ChaffIndistinguishability`)

Each layer must be evaluated separately. See ecosystem master FAQ.

---

## 4. Computational auxiliary in this repo

| Item | Type |
|------|------|
| Network timing, daemon scheduling | Operational |
| CLI parsing, file I/O | Glue |
| Git dependency resolution | Build |

---

## 5. Do not misread (auditors / AI)

1. **"ITS-routing uses hashing"** — Trace the subcommand; hashes may be build/AEH paths, not WC-MAC tags.

2. **"time-lock is fully ITS"** — RSW **squaring** is computational delay; the puzzle is **ITS** because RSW output $Y$ is **chained** into SSS ($s_{2,0} = (Y+m) \bmod p$). See timelock SECURITY_LAYERS.

3. **Rejecting ITS-routing for glue code** — Wrong unit of analysis; follow the table in §2.

---

## 7. Huller-lukning (UES v1.5)

| Hole | Closure | Doc / gate |
|------|---------|------------|
| **A — censur/sletning** | SSS over epochs, multi-courier, sneakernet | `AvailabilityResilience.lean`, `pipe_its_sneakernet_e2e.sh` |
| **O⁺ — metadata** | L3' constant harvest, fixed cell size | `MetadataSymmetry.lean`, no selective `--timeout-secs` in prod |
| **EP — endpoint** | Axiom + blast-radius | [ITS-routing_SecureEndpointDoctrine.md](ITS-routing_SecureEndpointDoctrine.md) |
| **Forkert implementation** | CertifiedBuild | `verify_ecosystem.sh` v1.5 gates, `rust_epoch_cell_refines_ideal` |

Master theorem: [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) — \(I(S;O_{\text{Eve}})=0\).

---

## 6. Cross-links

- [README.md](README.md)
- [ITS-routing_HEADS_UP.md](ITS-routing_HEADS_UP.md)
- [ITS-routing_mathematics.md](ITS-routing_mathematics.md)
- [ITS-routing_manual.md](ITS-routing_manual.md)
