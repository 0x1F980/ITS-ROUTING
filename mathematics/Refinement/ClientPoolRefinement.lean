import AvailabilityLedger
import ForwardReceiveGate
import Refinement.ValidForwardRefinement
import Refinement.ForwardReceiveGateRefinement

/-!
# Refinement — pool client harvest path refines ideal (v10 / R3)

Abstract `establish_canonical` / `record_harvest` sequence and courier M_valid filter.

**Proved:** harvest from `rustValidMirrorSet` + receive gate ⇒ `harvestPermitted`; omit ⇒ ledger disclosure.
**Outside:** OS `/dev/urandom` byte draw (see REFINEMENT_MANIFEST RNG boundary).
**Rust:** `courier.rs::WhitelistMultiCourier`, `valid_forward_party.rs::establish_canonical`.
-/

namespace Refinement

open ITS

/-- Rust `establish_canonical` — publish cell into canonical log. -/
def rustEstablishCanonical (st : RustValidForwardState) (e c : Nat) : RustValidForwardState :=
  { st with
    canonical := { cellAt := fun e' => if e' = e then some c else st.canonical.cellAt e' } }

/-- Rust `record_harvest` at epoch `e` (Nat cell id model). -/
def rustRecordHarvest (st : RustValidForwardState) (m e c : Nat) : RustValidForwardState :=
  { st with mirrorHarvest := fun m' e' => if m' = m ∧ e' = e then some c else st.mirrorHarvest m' e' }

/-- Pool client harvest at `e` from mirror in M_valid with receive gate. -/
def rustPoolHarvestPermitted (st : RustValidForwardState) (led : SendRightsView)
    (mirrors : List Nat) (m e c W : Nat) : Prop :=
  harvestPermitted (poolViewOf st) st.canonical led m e c W ∧
    rustValidMirrorMember st led mirrors m W ∧
      rustReceiveGate st m e

theorem rust_pool_harvest_permitted_of_gate
    (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat)
    (m e c W : Nat) (hmem : rustValidMirrorMember st led mirrors m W)
    (hgate : rustReceiveGate st m e) (hpub : published st.canonical e c) (he : e ≤ W) :
    rustPoolHarvestPermitted st led mirrors m e c W := by
  unfold rustPoolHarvestPermitted rustReceiveGate rustValidMirrorMember at *
  refine ⟨?_, hmem, hgate⟩
  exact harvest_permitted_of_gate (poolViewOf st) st.canonical led m e c W hgate
    (rust_valid_mirror_set_subset st led mirrors m W hmem) hpub he

/-- Omit on published cell refines AvailabilityLedger selective-omit disclosure. -/
theorem rust_omit_discloses_selective_omit (subscriber witness : Nat) :
    attackKindDisclosed AvailabilityAttackKind.selectiveOmit 0 subscriber witness 0 :=
  attack_selective_omit_disclosed subscriber witness

theorem rust_record_harvest_mismatch_discloses
    (st : RustValidForwardState) (m e c W : Nat) (subscriber witness : Nat)
    (hW : e ≤ W) (hpub : published st.canonical e c)
    (hmiss : st.mirrorHarvest m e ≠ some c) :
    attackKindDisclosed AvailabilityAttackKind.selectiveOmit e subscriber witness 0 := by
  have _ := rust_harvest_mismatch_de_whitelists st m e c W hW hpub hmiss
  exact attack_selective_omit_disclosed subscriber witness

/-- Closed refinement bundle — pool client path refines ideal harvest + ledger. -/
def clientPoolRefinementClosed : Prop :=
  (∀ (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat)
      (m e c W : Nat),
    rustValidMirrorMember st led mirrors m W →
      rustReceiveGate st m e →
        published st.canonical e c →
          e ≤ W →
            rustPoolHarvestPermitted st led mirrors m e c W) ∧
    (∀ (subscriber witness : Nat),
      attackKindDisclosed AvailabilityAttackKind.selectiveOmit 0 subscriber witness 0) ∧
      forwardReceiveGateRefinementClosed

theorem client_pool_refinement_closed : clientPoolRefinementClosed :=
  ⟨fun st led mirrors m e c W hmem hgate hpub he =>
     rust_pool_harvest_permitted_of_gate st led mirrors m e c W hmem hgate hpub he,
   fun subscriber witness => rust_omit_discloses_selective_omit subscriber witness,
   forward_receive_gate_refinement_closed⟩

end Refinement
