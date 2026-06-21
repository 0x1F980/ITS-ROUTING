# ITS Infrastructure Replacement — RSA/PQC/Tor → ITS (v5)

## License: GNU GPLv3 Only

**Read with:** [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md) · [ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md)

This document is the **official replacement matrix** for migrating from computational infrastructure (RSA, PQC, TLS record crypto, Argon2 vaults, Tor/Nym mixnets) to **Shannon Information-Theoretic Secrecy (ITS)** across four channels.

**Threat model:** Eve owns all network infrastructure (ISP, DNS, mixnodes, timing, volume). Alice and Bob operate from secure endpoints. Eve has **unbounded computational power**.

---

## Four ITS channels

| Channel | Property | Owner |
|---------|----------|-------|
| **C1 Payload** | \(I(M; C_{\text{wire}}) = 0\) — Eve cannot derive plaintext | ITS-asymmetric |
| **C2 Integriteit** | WC-MAC forgery bound \(P \le 1/p\) | ITS-OTM |
| **C3 Anonymitet** | \(I(\text{deanonym bits}; \text{observed traffic}) = 0\) under spec | ROUTING (its_transport) |
| **C4 Benægtelighed** | Under tvang: alternative plaintexts algebraisk konsistente | ITS-timelock L2 (SSS) |

**Aux (isolated, carries no secrecy):** RSW L1 squaring = time wall only.

---

## Replacement matrix

| Legacy infrastructure | ITS replacement | Channel | Status v1.1 |
|----------------------|-----------------|---------|-------------|
| RSA/PQC wire encrypt | `its_asymmetric encrypt` | C1 | Done |
| RSA/PQC forward secrecy | `its_asymmetric epoch-advance` | C1 | Done |
| Ed25519/RSA signatures (ecosystem) | `its_otm sign/verify` | C2 | Done |
| TLS 1.3 record AES-GCM | `its-wire/1` ALPN — **no** record encryption | C1 | Fase 3 |
| X.509 / Web PKI trust | OTM tier 1a/1b + KM contact graph | C2 | Fase 3 |
| Argon2 vault (ITSKMV2) | ITSKMV3 `.wire` via `its_asymmetric` | C1 | Fase 1 |
| Ledger Argon2 vault | ITS wire seal (same pattern) | C1 | Fase 1 |
| Tor/Nym onion + mix | **UES Monocell Pool** (`--pool` default) | C3 | **v1.5** |
| AEH last-resort (pool ban) | `client-send/receive --aeh` | C3/C4 | **v1.5** |
| Cover traffic / chaff | \(C_e \sim \mathcal{D}\) via `epoch_cell::step` | C3 | **v1.5** |
| HKDF `StateRatchet` | `TransportOtpRatchet` (SSS epoch) | C3 | Done |
| Dev onion/mix regression | `dev-onion-mix` feature | C3 dev | Legacy |
| Timelock manipulation | SSS underbestemmelse (L2) | C4 | Done |
| GnuPG/Kleopatra flow | `its-km` subprocess pipes | Glue | Done |

---

## UES vs Tor / Nym / I2P (v2.0 superiority)

Full win-condition matrix (W1–W13): **[ITS-routing_SUPERIORITY.md](ITS-routing_SUPERIORITY.md)**. Participation symmetry (O⁺): **[ITS-routing_PARTICIPATION_SYMMETRY.md](ITS-routing_PARTICIPATION_SYMMETRY.md)**.

| | Tor/Nym/I2P | **UES v2.0 (pool primary)** |
|---|-------------|-----------------------------|
| C/I | Computational | **ITS** (\(I(S;O)=0\) in \(O\)) |
| Sybil 98% nodes | Deanonymisering mulig | **C/I uændret** |
| Latency | Multi-hop / mix window | **1 epoch, 0 hops** |
| Idle leak | Mulig | **L3 konstant** |
| Operator UX | 1 router + proxy | **`its-km send/receive`** + optional `its-pool-proxy` |
| Modenhed | Shipped | **v2.0 — 9-pipe verify gate** |

---

## Migration: RSA/PQC wire

```bash
# Before (computational)
# openssl rsautl -encrypt -inkey bob.pub -pubin -in msg.txt -out msg.enc

# After (Shannon ITS)
its_asymmetric encrypt --pk bob.public.key --in msg.txt --out msg.wire
its_asymmetric decrypt --sk bob.secret.key --pk bob.public.key --in msg.wire --out msg.txt
its_asymmetric epoch-advance --pk bob.public.key --sk bob.secret.key   # forward secrecy
```

---

## Migration: TLS + RSA → ITS wire profile

See [ITS-asymmetric/docs/ITS_TLS_ALPN_DESIGN.md](../ITS-asymmetric/docs/ITS_TLS_ALPN_DESIGN.md).

| Today | ITS v1.1 |
|-------|----------|
| TLS 1.3 + cert RSA/ECDSA | ALPN `its-wire/1` + OTM contact attestation |
| Server decrypts with RSA privkey | Bob local `secret.key` only |
| AES-GCM record encryption | **None** — confidentiality is Shannon wire only |

```bash
ROUTING/scripts/its-curl.sh --pk bob.public.key --file doc.txt https://endpoint/
```

---

## Migration: Tor/Nym → ITS routing

```bash
# Wire ITS payload (opaque bytes into transport)
its_asymmetric encrypt --pk bob.public.key --in doc.txt --out - \
  | its-routing client-send --ratchet-seed-file seed.bin ...

# Full daemon with ridges
cargo build -p its_routing --release --features full
its-routing daemon --config config.toml
```

**Option B (passive):** AEH + steganography — see [ITS-routing_vision.md](ITS-routing_vision.md).

---

## Migration: Vault Argon2 → ITSKMV3

```bash
its-km vault init --vault-key-dir ~/.its/vault-keys
# Produces vault.public.key + vault.secret.key (local only) + km.vault (ITSKMV3 wire blobs)

its-km vault open --vault-secret ~/.its/vault-keys/vault.secret.key
its-km vault seal
```

**Rule:** `vault.secret.key` never ships inside `km.vault`. Offline thief sees only Shannon ITS ciphertext.

---

## Out of v1.1 scope

| Item | Reason |
|------|--------|
| Global Chrome root CA replacement | Ecosystem OTM only; not browser CA |
| FIPS 140 module labels | Parallel path — [FIPS_PARALLEL_PATH.md](../ITS-asymmetric/docs/FIPS_PARALLEL_PATH.md) |
| Nym token economics | Operator-deploy model |

---

## Verify

```bash
./scripts/verify_ecosystem.sh
./scripts/pipe_its_routing_e2e.sh
./scripts/bootstrap.sh
```
