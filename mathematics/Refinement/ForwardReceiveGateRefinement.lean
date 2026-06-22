import ForwardReceiveGate
import Refinement.ValidForwardRefinement

/-!
# Refinement — Rust `receive_gate` + courier M_valid filter (v10 / M25)

Abstract receive gate over `RustValidForwardState` embedding.

**Proved:** vacuous @0; valid mirror ⇒ gate; alternate-from-M_valid ⇒ `forwardProof`.
**Rust:** `valid_forward_party.rs::receive_gate`, `courier.rs::WhitelistMultiCourier`.
-/

namespace Refinement

open ITS

/-- Rust `receive_gate` — ValidFwd over `[0, e-1]`. -/
def rustReceiveGate (st : RustValidForwardState) (m e : Nat) : Prop :=
  receiveGate (poolViewOf st) st.canonical m e

theorem rust_receive_gate_vacuous_at_zero (st : RustValidForwardState) (m : Nat) :
    rustReceiveGate st m 0 :=
  receive_gate_vacuous_at_zero (poolViewOf st) st.canonical m

theorem rust_valid_mirror_gives_receive_gate
    (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat)
    (m e W : Nat) (hmem : rustValidMirrorMember st led mirrors m W)
    (he : e ≤ W + 1) :
    rustReceiveGate st m e := by
  unfold rustReceiveGate
  exact valid_mirror_gives_receive_gate (poolViewOf st) st.canonical led m e W
    (rust_valid_mirror_set_subset st led mirrors m W hmem) he

/-- Courier harvest filter: only mirrors in M_valid with receive gate. -/
def rustHarvestFromValidMirrorsOnly (st : RustValidForwardState) (led : SendRightsView)
    (mirrors : List Nat) (m e c W : Nat) : Prop :=
  rustValidMirrorMember st led mirrors m W ∧
    rustReceiveGate st m e ∧
      published st.canonical e c ∧
        st.mirrorHarvest m e = some c

/-- Rust alternate path from M_valid — mirrors `alternateFromValidMirrors`. -/
def rustAlternateFromValidMirrors (st : RustValidForwardState) (led : SendRightsView)
    (mirrors : List Nat) (s e c W : Nat) : Prop :=
  alternateFromValidMirrors (poolViewOf st) st.canonical led mirrors s e c W

theorem rust_alternate_from_valid_mirrors
    (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat) (s e c W : Nat) :
    rustAlternateFromValidMirrors st led mirrors s e c W ↔
      alternateFromValidMirrors (poolViewOf st) st.canonical led mirrors s e c W :=
  Iff.rfl

theorem rust_alternate_from_valid_mirrors_gives_forward_proof
    (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat) (s e c W : Nat)
    (h : rustAlternateFromValidMirrors st led mirrors s e c W) :
    forwardProof (poolViewOf st) st.canonical e c :=
  alternate_from_valid_mirrors_gives_forward_proof (poolViewOf st) st.canonical led mirrors s e c W h

theorem rust_harvest_from_valid_mirrors_gives_forward_proof
    (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat)
    (m s e c W : Nat) (h : rustHarvestFromValidMirrorsOnly st led mirrors m e c W) :
    forwardProof (poolViewOf st) st.canonical e c := by
  rcases h with ⟨hmem, _hgate, hpub, hharvest⟩
  rcases hmem with ⟨_, hvf, _⟩
  exact ⟨hpub, m, hharvest⟩

/-- Closed refinement bundle — Rust receive gate refines ideal. -/
def forwardReceiveGateRefinementClosed : Prop :=
  (∀ (st : RustValidForwardState) (m : Nat), rustReceiveGate st m 0) ∧
    (∀ (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat) (m e W : Nat),
      rustValidMirrorMember st led mirrors m W → e ≤ W + 1 → rustReceiveGate st m e) ∧
      (∀ (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat) (s e c W : Nat),
        rustAlternateFromValidMirrors st led mirrors s e c W →
          forwardProof (poolViewOf st) st.canonical e c) ∧
        forwardReceiveGateClosed

theorem forward_receive_gate_refinement_closed : forwardReceiveGateRefinementClosed :=
  ⟨fun st m => rust_receive_gate_vacuous_at_zero st m,
   fun st led mirrors m e W hmem he =>
     rust_valid_mirror_gives_receive_gate st led mirrors m e W hmem he,
   fun st led mirrors s e c W h =>
     rust_alternate_from_valid_mirrors_gives_forward_proof st led mirrors s e c W h,
   forward_receive_gate_closed⟩

end Refinement
