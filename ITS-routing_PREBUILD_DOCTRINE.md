# ITS-routing: Pre-Build Doctrine

**Status:** Forudsætning for [UES Pool + AEH v1.5](/home/user/.cursor/plans/ues_pool_+_aeh_bf88c516.plan.md).  
**Formål:** Dokumentér at økosystemet efter afkobling er byg-bart, verificeret og rent lagdelt — uden at implementere UES Pool endnu.

Se også: [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md) (constitution) · [ITS-routing_KEEP_BOUNDARY.md](ITS-routing_KEEP_BOUNDARY.md) (transport boundary).

---

## 1. Build-profiler

Default `its_routing` build er **transport only** — ridges er opt-in. Matcher composable infrastructure: tilføj kun det din pipeline behøver.

| Profile | Command | What you get |
|---------|---------|--------------|
| **Transport** (default) | `cargo build -p its_routing --release` | Onion, SSS fragment, AEH core, client-send/receive |
| **Full daemon** | `cargo build -p its_routing --release --features full` | Transport + alle operational ridges |
| **Custom** | `cargo build -p its_routing --release --features timelock,ledger` | Transport + valgte ridges |

| Feature | Enables |
|---------|---------|
| `full` | Alle ridges nedenfor (convenience bundle) |
| `hardware` | TRNG via ITS-hardware, analog share export/import |
| `ledger` | Live blockchain hash fetch for AEH |
| `timelock` | `time-lock`, `time-unlock`, `time-deny` subcommands |
| `otm` | OTM tag verification on AEH blocks |
| `fingerprint-erasure` | Γ on send + `fingerprint-erasure` subcommand |

Deaktiverede subcommands printer rebuild-hint: `cargo build -p its_routing --features <name>`.

Ecosystem verify bruger `--features full` når den tester den komplette routing-binary.

---

## 2. Pipe-afkobling

Transport **må ikke** Cargo-depende på `its_asymmetric`. Math-repos kører som **subprocess / pipe**, ikke som compile-deps.

| Concern | Mechanism |
|---------|-----------|
| Wire seal (ITS-asymmetric) | `ITS_ASYMMETRIC_DIR` + `ITS_ASYMMETRIC_BIN` — subprocess encrypt/decrypt |
| OTP ratchet seed | `its-km export-ratchet-seed` → `--ratchet-seed-file` på send/receive |
| OTM attestation | `its_otm` CLI pipe (feature `otm`) |
| Timelock | `its_timelock` / in-process ridge (feature `timelock`) |
| Operator identity | **ITS-KeyManagement only** — routing ser aldrig passwords |

Verifikation:

```bash
cargo tree -p its_routing 2>/dev/null | grep -E 'its_asymmetric|core_logic'  # skal være tomt
./scripts/verify_ecosystem.sh /path/to/ecosystem-root
```

---

## 3. Forbud i prod-claim

Fra [ITS-routing_KEEP_BOUNDARY.md](ITS-routing_KEEP_BOUNDARY.md):

| Path | Prod? | Rule |
|------|-------|------|
| AEH **uden** `--ratchet-seed-file` | **Nej** | Lab/demo — `demo_aeh_seed` fra `[crypto]` anchor+whitening |
| `demo_aeh_seed` i `ratchet.rs` | **Nej** | Forudsigelig fra config; kun CI/lokale demos |
| UDP onion default (`pipe_its_routing_e2e.sh`) | **Demo** | Indtil UES Fase 2 — ikke UES Monocell Pool |
| `WIKI_STEGO:` simulated AEH channel | **Demo** | Ingen rigtig stego-embed; se §5 |

**Production AEH flow:**

```bash
its-km export-ratchet-seed --out /tmp/seed.bin --password '...' [--duress]
its-routing client-send -c config.toml -f payload.bin -d 2 --aeh --ratchet-seed-file /tmp/seed.bin
```

---

## 4. Kendte gaps før UES v1.5

Disse er **dokumenterede bevidste huller** — ikke skjulte demo-paths i prod-claim. Hard fail på modulus kommer i UES Fase 1.

### 4.1 Lean / Rust modulus mismatch

| Location | Modulus | Note |
|----------|---------|------|
| `mathematics/Transport/Basic.lean` | `65537` | Lean transport proof baseline |
| `its_transport` (Rust) | `2147483647` (`2^31 - 1`) | Mersenne-31 felt i wire/daemon |

**UES krav:** Lean og Rust skal matche (`2147483647`). Rettes i UES v1.5 Fase 1 — **ikke** i denne pre-build plan.

**Verify-gate (advarsel, ikke fail):**

```bash
# Pre-build: dokumenteret mismatch — warn only
grep 'fieldPrime' mathematics/Transport/Basic.lean   # forvent 65537
grep -r '2147483647' its_transport/src/field_arith.rs  # forvent Mersenne-31
```

### 4.2 UDP-default transport (ikke UES Pool)

- Nuværende prod-demo path: direkte UDP SSS onion mellem noder (`start-node`, `client-send`/`client-receive`).
- `pipe_its_routing_e2e.sh` validerer denne UDP-sti — **ikke** broadcast pool / epoch cells.
- UES Monocell Pool-Net erstatter dette i UES v1.5 Fase 2.

### 4.3 `WIKI_STEGO:` simulated AEH

- `its_routing/src/aeh_channel.rs` bruger prefix `WIKI_STEGO:enwiki;...` som **simuleret** offentlig entropy-kanal.
- Ingen rigtig Wikipedia-steganografi; lab/demo for AEH winnowing-flow.
- UES v1.5 definerer rigtig pool-broadcast + AEH last-resort.

---

## 5. Pre-build checkliste (8 gates)

Kør fra ecosystem root (fx `/home/user`):

| # | Gate | Command / check |
|---|------|-----------------|
| 1 | Ecosystem verify | `./ROUTING/scripts/verify_ecosystem.sh /home/user` → `ALL CHECKS PASSED` |
| 2 | No forbidden deps | `cargo tree -p its_routing` uden `its_asymmetric` / `core_logic` |
| 3 | Default build | `cargo build -p its_routing --release` grøn |
| 4 | Full test | `cargo test -p its_transport -p its_routing --features full` grøn |
| 5 | Doctrine doc | Denne fil findes + linket fra `ITS_ECOSYSTEM.md` §Build & verify |
| 6 | Completions | Ingen ghost subcommands (`status-audit`, `verify-path`, `list-peers`, `--daemonize`) |
| 7 | Dead config | `clue_offset` fjernet fra `config.rs` og eksempler |
| 8 | Gaps documented | UDP demo, `WIKI_STEGO`, modulus mismatch — §4 ovenfor |

Når alle 8 er grønne: **klar til at starte UES v1.5** — ingen v1.5 tag i denne fase.

---

## 6. Næste skridt

Start [UES Pool + AEH v1.5](/home/user/.cursor/plans/ues_pool_+_aeh_bf88c516.plan.md):

1. Fase 0 — `UNATTACKABLE_MODEL` (math)
2. Fase 1 — Lean/Rust modulus alignment (hard fail gate)
3. Fase 2 — Pool-kode, `epoch_cell.rs`, `PoolCourier`
4. CertifiedBuild gates + `pipe_its_pool_e2e.sh`

**Denne plan implementerer ikke:** UES Pool, daemon-omskrivning, huller-lukning (MetadataSymmetry, ParticipationTheorem), eller erstatning af `pipe_its_routing_e2e.sh`.
