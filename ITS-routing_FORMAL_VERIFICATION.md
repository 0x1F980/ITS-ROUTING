# ITS-routing — formal verification (W6)

## License: GNU GPLv3 Only

---

## Gate matrix (cert / refinement / smoke)

| Target | Script / command | Lean lib / Rust features | Scope |
|--------|------------------|--------------------------|-------|
| **Math certificate** | `./scripts/verify_math.sh` | `routing-math-cert` | M1–M20: v9 ideal, 0 `sorry`, no dev-onion in cert closure |
| **Implementation refinement (theorem)** | `./scripts/verify_math.sh` | `routing-math-refinement` | M23–M26: v10 cert, ITS-A refinement modules |
| **Dev-onion regression** | `./scripts/verify_ecosystem.sh` (dev jobs) | `routing-math-dev`; `cargo build -p its_routing --features dev-onion-mix` | MixAnonymity, ChaffIndistinguishability; onion mesh pipes |
| **Ecosystem smoke** | `./scripts/verify_ecosystem.sh` | M17 build + M21–M22 pipes | cargo tests, E2E regression — **not primary proof after v10** |
| **Prod binary (pool only)** | `cargo build -p its_routing --no-default-features --features pool` | No `onion`/`daemon`/`packet` symbols | UES Monocell Pool operator path |

**Rules:**
- Master unattackability = Lean cert path first (M1–M20 ideal + M23–M26 refinement).
- E2E pipes (M18–M22) are smoke/regression only after v10.
- Dev-onion scripts and tests require explicit `--features dev-onion-mix`.
- Pool pipes share boilerplate via `scripts/lib/pipe_pool_common.sh`.
- Optional strict mode: `VERIFY_MATH_V10=1 ./scripts/verify_ecosystem.sh` runs full M1–M26.

---

## Three gates (math / refinement / smoke)

| Gate | Script | Scope |
|------|--------|-------|
| **Math certificate (ideal v9)** | `./scripts/verify_math.sh` M1–M20 | Lean only — `lake build routing-math-cert`, 0 `sorry` |
| **Implementation refinement (v10)** | `./scripts/verify_math.sh` M23–M26 | `routing-math-refinement`, `networkImplementationCertificateV10` |
| **Ecosystem smoke** | `./scripts/verify_ecosystem.sh` | cargo, M17 build, M21–M22 pipes (regression) |

**Rule:** Master unattackability is sealed in Lean first. Rust must refine ideal — drift fails M23–M26.

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

## Refinement gates (phase 3 — v10 theorem + phase 2 smoke)

```bash
cd ROUTING && ./scripts/verify_math.sh          # M1–M26 (ideal + refinement)
cd ROUTING/mathematics && lake build routing-math-refinement
cd ROUTING && cargo test -p its_routing --lib valid_forward consensus --quiet
./scripts/verify_ecosystem.sh /home/user        # M17 + M21–M22 smoke
```

### Subcommand → proof map (implementation)

| `its-routing` path | Upstream kernel | Refinement status |
|----------------|-----------------|-------------------|
| Transport OTP ratchet | `transport_otp_ratchet` + SSS epoch | **Proved (Lean)** — `Transport/RatchetDerivation.lean` |
| UES epoch cell / pool | `epoch_cell.rs` | **Proved (counter + support)** — `Refinement/EpochCellCorrectness.lean` |
| ValidFwd / M_valid | `valid_forward_party.rs` | **Proved** — `Refinement/ValidForwardRefinement.lean` |
| Witness k-of-n | `witness_consensus.rs` | **Proved** — `Refinement/WitnessConsensusRefinement.lean` |
| Receive gate / courier | `courier.rs` | **Proved** — `Refinement/ForwardReceiveGateRefinement.lean` |
| Pool client harvest | `courier.rs` receive | **Proved** — `Refinement/ClientPoolRefinement.lean` |
| End-to-end pool path | `client.rs` | **E2E smoke** — M21–M22 pipes |
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
