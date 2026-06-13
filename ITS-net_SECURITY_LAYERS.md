# ITS-net: Security Layers & ITS Scope (ITS-net_SECURITY_LAYERS)

## License: GNU GPLv3 Only
## Target: Auditors, AI-assisted reviewers, integrators

**Read this document first** before auditing this CLI/daemon repository.

**Ecosystem master:** [ITS_SECURITY_LAYERS.md](https://github.com/0x1F464/ITS/blob/master/ITS_SECURITY_LAYERS.md) (ITS core repo).

---

## 1. Role

ITS-net **orchestrates** upstream crates. It does not replace their proofs — it routes operators to the correct **per-layer** scope document.

---

## 2. Subcommand → upstream ITS layers

| Subcommand / flag | Upstream crate | ITS scope doc |
|-------------------|----------------|---------------|
| Default send stack (Γ + OTP + chaff) | ITS + ITS-fingerprint_erasure | [ITS_SECURITY_LAYERS.md](https://github.com/0x1F464/ITS/blob/master/ITS_SECURITY_LAYERS.md), [ITS-fingerprint_erasure_ITS_SCOPE.md](https://github.com/0x1F464/ITS-fingerprint_erasure/blob/master/ITS-fingerprint_erasure_ITS_SCOPE.md) |
| `--strict-stack` | FE + ITS wire stack | FE scope + core ITS §3 |
| `time-lock` / `time-unlock` / `time-deny` | ITS-self_enclosed_timelock | [ITS-self_enclosed_timelock_SECURITY_LAYERS.md](https://github.com/0x1F464/ITS-self_enclosed_timelock/blob/master/ITS-self_enclosed_timelock_SECURITY_LAYERS.md) |
| OTM / AEH attestation verify | ITS-OTM_public_attestation | [ITS-OTM_SECURITY_LAYERS.md](https://github.com/0x1F464/ITS-OTM_public_attestation/blob/main/ITS-OTM_SECURITY_LAYERS.md) |
| `--fingerprint-erasure` | ITS-fingerprint_erasure | [ITS-fingerprint_erasure_ITS_SCOPE.md](https://github.com/0x1F464/ITS-fingerprint_erasure/blob/master/ITS-fingerprint_erasure_ITS_SCOPE.md) |
| Hardware entropy hooks | ITS-hardware | [ITS-hardware_SECURITY_LAYERS.md](https://github.com/0x1F464/ITS-hardware/blob/master/ITS-hardware_SECURITY_LAYERS.md) |
| Ledger / vault sync | ITS-ledger | [ITS-ledger_SECURITY_LAYERS.md](https://github.com/0x1F464/ITS-ledger/blob/master/ITS-ledger_SECURITY_LAYERS.md) |

---

## 3. Default stack (documentation)

When operators use the full offline send path without disabling layers:

1. **Γ** — fingerprint erasure (discrete ITS subset + axiomatic continuous rules)
2. **OTP** — $C=M+K$ over $\mathbb{F}_p$ (ITS given uniform keys)
3. **Chaff** — traffic shaping (operational; not a substitute for OTP ITS)

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

1. **"ITS-net uses hashing"** — Trace the subcommand; hashes may be build/AEH paths, not WC-MAC tags.

2. **"time-lock is fully ITS"** — RSW delay is **computational**; SSS deniability is **ITS** (timelock SECURITY_LAYERS).

3. **Rejecting ITS-net for glue code** — Wrong unit of analysis; follow the table in §2.

---

## 6. Cross-links

- [README.md](README.md)
- [ITS-net_HEADS_UP.md](ITS-net_HEADS_UP.md)
- [ITS-net_mathematics.md](ITS-net_mathematics.md)
- [ITS-net_manual.md](ITS-net_manual.md)
