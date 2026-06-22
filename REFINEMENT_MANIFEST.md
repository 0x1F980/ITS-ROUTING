# ROUTING — Refinement manifest (phase 3 — v10 implementation certificate)

**Math certificate (phase 1):** [PROOF_MANIFEST.md](PROOF_MANIFEST.md) · `./scripts/verify_math.sh` M1–M26  
**Implementation certificate (phase 3):** `networkImplementationCertificateV10` in [`MasterTheoremV6.lean`](mathematics/MasterTheoremV6.lean)  
**Refinement gate (theorem):** M23–M26 in `./scripts/verify_math.sh`  
**Ecosystem gate (smoke):** `./scripts/verify_ecosystem.sh` — M17 + M21–M22 pipes (not primary proof)

**Ship blocker X4:** Rust must not silently diverge from Lean ideal — drift is a **math failure** (M23–M26).

---

## Status (v10 — Sprint R1–R5)

| Gate | Command | Status |
|------|---------|--------|
| **M23** Lean refinement lib (all v10 roots) | `lake build routing-math-refinement` | **Green** |
| **M24** ValidFwd refinement smoke | `Refinement/ValidForwardRefinement.lean` | **Green** |
| **M25** Witness + receive gate refinement smoke | `WitnessConsensusRefinement` + `ForwardReceiveGateRefinement` | **Green** |
| **M26** v10 cert + PROOF_MANIFEST grep | `MasterTheoremV6.lean` + manifest | **Green** |
| **M17** Ecosystem refinement build | `verify_ecosystem.sh` | **Green** |
| **M21–M22** E2E pipes + manifest | smoke only (not primary proof) | **Green** |
| ITS-A Rust unit tests | `cargo test -p its_routing --lib valid_forward consensus` | **Green** |

---

## Lean ↔ Rust map (v10)

| Concern | Lean ideal | Lean refinement | Rust implementation | Status |
|---------|------------|-----------------|----------------------|--------|
| OTP ratchet step | `Transport/RatchetDerivation.lean` | `EpochCellCorrectness.lean` | `transport_otp_ratchet` | **Proved** |
| UES epoch cell | `Transport/Epoch.lean`, `Cell.lean` | `EpochCellCorrectness.lean` | `epoch_cell.rs` | **Proved (counter + support)** |
| L1 cell bytes | `cellIndistinguishability` | support membership only | uniform RNG in `step()` | **Outside RNG** |
| ValidFwd / M_valid | `ValidForwardParty.lean` | `ValidForwardRefinement.lean` | `valid_forward_party.rs` | **Proved** |
| Witness k-of-n | `WitnessConsensus.lean` | `WitnessConsensusRefinement.lean` | `witness_consensus.rs` | **Proved** |
| Forward receive gate | `ForwardReceiveGate.lean` | `ForwardReceiveGateRefinement.lean` | `receive_gate`, `WhitelistMultiCourier` | **Proved** |
| Pool client harvest | `harvestPermitted` | `ClientPoolRefinement.lean` | `courier.rs` receive path | **Proved** |
| Omit → ledger strike | `AvailabilityLedger.lean` | `ClientPoolRefinement.lean` | `omit_de_whitelists_mirror` | **Proved (selectiveOmit disclosure)** |
| SSS fragment wire | `SSSMultiIPCourier.lean` | `SssWireRefinement.lean` (stub) | interleave roundtrip test | **Planned v10.1** |
| End-to-end binary | composition lemmas | — | `client.rs` pool/AEH | **E2E smoke (M18–M22)** |
| OTM tag verify | `IntegrityAxiom.lean` | — | `epoch_cell`, `aeh.rs` | **Cross-repo (ITS-OTM)** |
| SOCKS / KM glue | L3 + BIS | — | proxy, `its-km` | **E2E smoke** |

---

## v10.1 sibling refinement tracks (planned)

| Repo | Target | Lean location | Status |
|------|--------|---------------|--------|
| **ITS-asymmetric** | `fullWireEncShannonIts` execution | `mathematics/refinement/` (planned) | **Planned** |
| **ITS-OTM** | `verify_cell` ↔ OTM tag | extend OTM mathematics | **Planned** |
| **ITS-timelock** | ridge `timelock.rs` ↔ `TimelockComposition` | `stl/refinement/` (planned) | **Planned** |
| **SSS_CHAIN** | fragment interleave roundtrip | `Refinement/SssWireRefinement.lean` | **Stub / planned** |

`verify_ecosystem.sh` runs `lake build *-refinement` when sibling lakefile defines a `*-refinement` lib.

---

## What is proved vs smoke vs Outside

### Proved in Lean (`Refinement/` — 0 `sorry`, 0 `Prop := True`)

- **Epoch cell:** `rust_epoch_counter_refines_ideal`, `epochCellRefinementClosed`
- **ValidFwd:** `rust_omit_de_whitelists`, `rust_valid_forward_party_sound`, `validForwardRefinementClosed`
- **Witness:** `rust_consensus_at_epoch_iff`, `rust_count_ge_gives_consensus`, `witnessConsensusRefinementClosed`
- **Receive gate:** `rust_receive_gate_vacuous_at_zero`, `forwardReceiveGateRefinementClosed`
- **Client pool:** `rust_pool_harvest_permitted_of_gate`, `clientPoolRefinementClosed`
- **Master v10:** `network_implementation_certificate_v10`

### E2E smoke (regression only)

- M18–M22 pipes: pool, SOCKS, KM, timelock, censorship recovery
- `cargo test` integration paths

### Outside (explicit boundary)

| Layer | Notes |
|-------|-------|
| **RNG / OS entropy** | Cell byte draw uniform in Rust; Lean proves tag ∈ F_p support only (option B) |
| M31 field ring | Cross-repo axioms in `ItsMath.Field` |
| Side-channels, both-EP compromised | Documented Outside |
| SSS wire byte-for-byte | Operational roundtrip test until v10.1 |
| dev-onion mix | Out of cert/refinement (M16) |

---

## Commands

```bash
# Phase 1 + 3 math gate (M1–M26)
cd ROUTING && ./scripts/verify_math.sh

# Refinement lib only (M23)
cd ROUTING/mathematics && lake build routing-math-refinement

# ITS-A Rust unit tests
cd ROUTING && cargo test -p its_routing --lib valid_forward consensus --quiet

# Ecosystem smoke (M17, M21–M22)
cd ROUTING && ./scripts/verify_ecosystem.sh /home/user

# Strict v10 math on ecosystem run
VERIFY_MATH_V10=1 ./scripts/verify_ecosystem.sh /home/user
```

---

## Related docs

- [PROOF_MANIFEST.md](PROOF_MANIFEST.md) — v10 one-liner + phase 3 row
- [ITS-routing_MATHEMATICAL_CORE.md](ITS-routing_MATHEMATICAL_CORE.md) — §0c MathSupremacy, §Refinement
- [ITS-routing_FORMAL_VERIFICATION.md](ITS-routing_FORMAL_VERIFICATION.md) — three-gate matrix

**Constitution:** [ITS_ECOSYSTEM.md](ITS_ECOSYSTEM.md)
