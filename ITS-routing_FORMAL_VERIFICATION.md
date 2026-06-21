# ITS-routing — formal verification (W6)

## License: GNU GPLv3 Only

---

## Gate matrix (cert / dev / ecosystem)

| Target | Script / command | Lean lib / Rust features | Scope |
|--------|------------------|--------------------------|-------|
| **Math certificate** | `./scripts/verify_math.sh` | `routing-math-cert` (transitive from `UnattackableCertificate`) | M1–M8; 0 `sorry`; no dev-onion in cert closure |
| **Dev-onion regression** | `./scripts/verify_ecosystem.sh` (dev jobs) | `routing-math-dev`; `cargo build -p its_routing --features dev-onion-mix` | MixAnonymity, ChaffIndistinguishability; onion mesh pipes |
| **Refinement / ecosystem** | `./scripts/verify_ecosystem.sh` | `routing-math-refinement`; default `pool` Rust | cargo tests, pool E2E pipes, `EpochCellCorrectness` |
| **Prod binary (pool only)** | `cargo build -p its_routing --no-default-features --features pool` | No `onion`/`daemon`/`packet` symbols | UES Monocell Pool operator path |

**Rules:**
- Master unattackability = Lean cert path first; Rust is refinement (phase 2).
- Dev-onion scripts and tests require explicit `--features dev-onion-mix`.
- Pool pipes share boilerplate via `scripts/lib/pipe_pool_common.sh`.

---

## Two gates (math vs refinement)

| Gate | Script | Scope |
|------|--------|-------|
| **Math certificate** | `./scripts/verify_math.sh` | Lean only — `lake build`, 0 `sorry`, `UnattackableCertificate.lean` |
| **Refinement / ecosystem** | `./scripts/verify_ecosystem.sh` | cargo, E2E pipes, Rust ↔ ideal (phase 2) |

**Rule:** Master unattackability is sealed in Lean first. No Rust change required for math gate green.

---

## Math certificate (phase 1 — sealed)

```bash
cd ROUTING && ./scripts/verify_math.sh
```

| Claim | Lean module | Math status |
|-------|-------------|-------------|
| Master unattackable certificate | `UnattackableCertificate.lean` | **Proved** |
| C1 wire Shannon | `Transport/WireComposition.lean` + `ITS-asymmetric` | **Proved** |
| C2 integrity | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` | **Proved** (OTM import) |
| C3 stream + Sybil + MathSupremacy | `UnifiedEpochStream.lean`, etc. | **Proved** (finite-MI) |
| I(author; O) = 0 | `AuthorAttributionZero.lean` | **Proved** |
| O⁺ under P1–P3 | `OplusClosure.lean` | **Postulate-under-P1–P3** |
| Offline O_net = ∅ | `OfflineChannel.lean` | **Proved** |
| EP split | `EndpointSplit.lean` | **Proved** |
| Observation O / O⁺ / O_phys | `ObservationAlphabet.lean` | **Proved** (scope formalized) |

Full lemma map: [PROOF_MANIFEST.md](PROOF_MANIFEST.md) v4 (v4 MI status column) · [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) v5

---

## Refinement gates (phase 2 — software/hardware)

```bash
cd its_routing && cargo test
./scripts/verify_ecosystem.sh /home/user
```

### Subcommand → proof map (implementation)

| `its-routing` path | Upstream kernel | Refinement status |
|----------------|-----------------|-------------------|
| Transport OTP ratchet | `transport_otp_ratchet` + SSS epoch | **Proved (Lean)** — `Transport/RatchetDerivation.lean` |
| UES epoch cell / pool | `epoch_cell.rs` | **Proved (counter + support)** — `Refinement/EpochCellCorrectness.lean`; see [REFINEMENT_MANIFEST.md](REFINEMENT_MANIFEST.md) |
| Mode P ⊗ AEH composition | pool + AEH client paths | **E2E pipes** + Lean `Transport/Composition.lean` |
| Size-independent (N=1) | FewUser + Participation | **Proved (Lean)** |
| ITS chaff indistinguishability | `create_chaff_onion_packet` | **Proved (Lean)** — dev-onion-mix |
| Morphic mix anonymity | `blind_linear_mix` | **Proved (Lean)** |
| strict stack send (Γ + OTP + chaff) | `composed_send_roundtrip` | **Proved** |
| `--fingerprint-erasure` | `its_fingerprint_erasure` strict stack | **Proved (Rust)** |
| OTM verify | W3 `rust_otm_refines_ideal` | **Proved** |
| time-lock / unlock / deny | timelock `stl/` W4.3 | **Proved** |
| time-deny OTP layer | `time_deny_otp_layer` | **Proved** |

---

## Related ITS crates (this workspace)

| Crate | Role | Verification doc |
|-------|------|------------------|
| [ITS-asymmetric](https://github.com/0x1F980/ITS-asymmetric) | Wire v6 static broadcast encrypt — **C1 source** | `ITS-asymmetric_FORMAL_VERIFICATION.md` |
| [ITS-fingerprint_erasure](https://github.com/0x1F980/ITS-FINGERPRINT_ERASURE) | Γ extended / two-domain NF | `ITS-fingerprint_erasure_FORMAL_VERIFICATION.md` |
| [ITS-self_enclosed_timelock](https://github.com/0x1F980/ITS-self_enclosed_timelock) | Time-lock puzzles | upstream W4 |

Lean composition (archived upstream): [ItsNet/Composition.lean](https://github.com/0x1F464/ITS/tree/master/mathematics/ItsMath/ItsNet/Composition.lean) — see [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)

---

## Rust tests (this repo)

| Claim | Test / command |
|-------|----------------|
| CLI builds | `cargo build --release` in `its_routing/` |
| Unit tests | `cargo test` in `its_routing/` |
| Timelock / OTM glue | `tests/timelock_integration.rs`, `tests/otm_verify_integration.rs` |
| Full ecosystem | `./scripts/verify_ecosystem.sh` |

See also: [ITS-routing_SECURITY_LAYERS.md](ITS-routing_SECURITY_LAYERS.md), [ITS-routing_mathematics.md](ITS-routing_mathematics.md).
