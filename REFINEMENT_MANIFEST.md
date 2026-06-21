# ROUTING — Refinement manifest (phase 2 — software ↔ Lean ideal)

**Math certificate (phase 1):** [PROOF_MANIFEST.md](PROOF_MANIFEST.md) · `./scripts/verify_math.sh`  
**Refinement gate (phase 2):** `./scripts/verify_ecosystem.sh` — M17 + Rust tests  
**Formal spec:** [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) · [ITS-routing_FORMAL_VERIFICATION.md](ITS-routing_FORMAL_VERIFICATION.md)

**Ship blocker X4:** Rust must not silently diverge from Lean ideal without a green refinement gate (M17).

---

## Status (Sprint 4 — ca308ef5)

| Gate | Command | Status |
|------|---------|--------|
| **M17** Lean refinement lib | `lake build routing-math-refinement` | **Green** |
| **M7** epoch cell Rust test | `cargo test -p its_transport rust_epoch_cell_refines_ideal` | **Green** |
| Ratchet algebra test | `cargo test -p its_transport rust_ratchet_algebra_matches_lean` | **Green** |
| Ecosystem | `./scripts/verify_ecosystem.sh` | **Green** |

**Closes:** X4 (implementation drift ship blocker), M17 (refinement build gate).

---

## Lean ↔ Rust map

| Concern | Lean (ideal) | Rust (implementation) | Refinement status |
|---------|--------------|-------------------------|-------------------|
| OTP ratchet step | `Transport/RatchetDerivation.lean` — `ratchetStep`, `epochStepForward` | `its_transport::transport_otp_ratchet` | **Proved** — counter + k_pool by rfl; Rust test mirrors forward algebra |
| UES epoch cell | `Refinement/EpochCellCorrectness.lean` | `its_transport::epoch_cell` | **Proved (counter + support)** — epoch index = `idealStep.1`; cell tag ∈ F_p; fixed size L in Rust test |
| L1 cell indistinguishability | `Transport/Cell.lean` — `cellIndistinguishability` | uniform RNG fill in `step()` | **Proved (Lean)** — uniform draw over F_p; payload/chaff header not in O model |
| OTM tag verify | `IntegrityAxiom.lean` → `Otm.OtmIntegrity` | `epoch_cell::verify_cell`, `aeh.rs` | **Cross-repo** (ITS-OTM) |
| Pool client path | `UnifiedEpochStream.lean`, `MasterTheorem.lean` | `its_routing::client` pool receive | **E2E pipes** — `pipe_its_pool_e2e.sh` |
| End-to-end binary | composition lemmas | `client.rs` pool/AEH | E2E pipes in `verify_ecosystem.sh` |

---

## What is proved vs tested

### Proved in Lean (`Refinement/EpochCellCorrectness.lean`)

- `rust_epoch_counter_refines_ideal` — after step at epoch `e`, counter is `e + 1` (= `idealStep 0 e`).1).
- `rust_k_pool_matches_forward` — pool key equals `epochStepForward` algebra.
- `rust_ratchet_refines_ideal` — counter strictly increases each step.
- `rust_cell_tag_in_support` — draw mod `fieldPrime` lies in ideal distribution support.
- `epoch_cell_correctness` — bundles counter refinement + L1 `cellIndistinguishability`.

**Not proved by rfl alias:** `rustStep` is **not** defined as `idealStep`. Ideal L3 uses abstract `(e+1, e % p)`; Rust uses ratchet-derived keys plus uniform byte draw — refinement is counter alignment + support membership, not byte-for-byte equality.

### Proved in Rust tests

| Test | File | Claim |
|------|------|-------|
| `rust_ratchet_algebra_matches_lean` | `transport_otp_ratchet.rs` | `epoch_step_forward` = `current + anchor + counter + entropy` (M31 field) |
| `rust_epoch_cell_refines_ideal` | `epoch_cell.rs` | fixed cell size L; counter advances 0→1; deterministic replay across replicas |
| `otp_ratchet_stepping` | `transport_otp_ratchet.rs` | Alice/Bob ratchet sync |

---

## Axiom boundary (honest)

| Layer | Boundary | Notes |
|-------|----------|-------|
| M31 field ring | `ItsMath.Field.Basic` — `feAdd_assoc`, `feAdd_sub_cancel`, `feSub_add_cancel` | Axioms match Rust `field_arith` reduction; not re-proved in ROUTING refinement lib |
| RNG / OS entropy | Outside Lean | Cell bytes drawn uniformly in Rust; Lean models support `fieldPrime` only |
| SSS fragment wire | Operational | Roundtrip via `epoch_cell_sss_interleave_roundtrip` test |
| Both EP compromised | `OutsideChannel` | Not a refinement failure — outside theorem scope |

---

## Commands

```bash
# Lean refinement lib (M17)
cd ROUTING/mathematics && lake build routing-math-refinement

# Rust refinement tests
cd ROUTING && cargo test -p its_transport rust_epoch_cell_refines_ideal rust_ratchet_algebra_matches_lean --quiet

# Full phase-2 gate
cd ROUTING && ./scripts/verify_ecosystem.sh /home/user
```

---

## Related docs

- [PROOF_MANIFEST.md](PROOF_MANIFEST.md) — phase 2 table (refinement row)
- [ITS-routing_UNATTACKABLE_MODEL.md](ITS-routing_UNATTACKABLE_MODEL.md) — H4 implementation drift
- [ITS-routing_FORMAL_VERIFICATION.md](ITS-routing_FORMAL_VERIFICATION.md) — gate matrix

**Constitution:** [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)
