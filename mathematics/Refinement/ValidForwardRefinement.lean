import ValidForwardParty
import ForwardProof

/-!
# Refinement — Rust `valid_forward_party` refines ideal ValidFwd (v10 / M24)

Abstract Nat-indexed mirror model of `its_routing::valid_forward_party` — no Rust import.

**Proved:** de-whitelist ⇒ ¬ValidFwd; soundness vs ideal `validForwardParty`; M_valid subset.
**Rust:** `valid_forward_party.rs` — `record_harvest`, `omit_de_whitelists_mirror`, `valid_mirror_set`.
-/

namespace Refinement

open ITS

/-- Abstract Rust `ValidForwardState`: canonical log + per-mirror harvest + de-whitelist set. -/
structure RustValidForwardState where
  canonical : CanonicalLog
  mirrorHarvest : Nat → Nat → Option Nat
  deWhitelisted : Nat → Prop

/-- Embed Rust harvest map into ideal `PoolView`. -/
def poolViewOf (st : RustValidForwardState) : PoolView :=
  ⟨st.mirrorHarvest⟩

/-- Rust `omit_de_whitelists_mirror` — mark mirror de-whitelisted. -/
def rustOmitDeWhitelist (st : RustValidForwardState) (m : Nat) : RustValidForwardState :=
  { st with deWhitelisted := fun a => a = m ∨ st.deWhitelisted a }

/-- Rust `valid_forward_party`: not de-whitelisted and valid forward history in window. -/
def rustValidForwardParty (st : RustValidForwardState) (m W : Nat) : Prop :=
  ¬ st.deWhitelisted m ∧
    validForwardParty (poolViewOf st) st.canonical m W

/-- Rust `valid_mirror_set` member: ValidFwd + send rights intact. -/
def rustValidMirrorMember (st : RustValidForwardState) (led : SendRightsView)
    (mirrors : List Nat) (m W : Nat) : Prop :=
  m ∈ mirrors ∧
    rustValidForwardParty st m W ∧
      ¬ led.revoked m

/-- Rust-side mirror whitelist (list-level). -/
def rustValidMirrorSet (st : RustValidForwardState) (led : SendRightsView)
    (mirrors : List Nat) (W : Nat) : Prop :=
  ∀ m, rustValidMirrorMember st led mirrors m W → m ∈ mirrors

theorem rust_omit_de_whitelists (st : RustValidForwardState) (m W : Nat)
    (hde : st.deWhitelisted m) : ¬ rustValidForwardParty st m W :=
  fun ⟨hnot, _⟩ => hnot hde

theorem rust_harvest_mismatch_de_whitelists
    (st : RustValidForwardState) (m e c W : Nat)
    (hW : e ≤ W) (hpub : published st.canonical e c)
    (hmiss : st.mirrorHarvest m e ≠ some c) :
    invalidForwardParty (poolViewOf st) st.canonical m W :=
  omit_de_whitelists_mirror (poolViewOf st) st.canonical m e c W hW hpub hmiss

theorem rust_valid_forward_party_sound (st : RustValidForwardState) (m W : Nat)
    (h : rustValidForwardParty st m W) :
    validForwardParty (poolViewOf st) st.canonical m W :=
  h.2

theorem rust_valid_mirror_set_subset
    (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat) (m W : Nat)
    (hmem : rustValidMirrorMember st led mirrors m W) :
    validMirror (poolViewOf st) st.canonical led m W :=
  ⟨rust_valid_forward_party_sound st m W hmem.2.1, hmem.2.2⟩

/-- Closed refinement bundle — Rust ValidFwd ops refine ideal lemmas. -/
def validForwardRefinementClosed : Prop :=
  (∀ (st : RustValidForwardState) (m W : Nat), st.deWhitelisted m →
      ¬ rustValidForwardParty st m W) ∧
    (∀ (st : RustValidForwardState) (m e c W : Nat),
      e ≤ W → published st.canonical e c → st.mirrorHarvest m e ≠ some c →
        invalidForwardParty (poolViewOf st) st.canonical m W) ∧
      (∀ (st : RustValidForwardState) (m W : Nat),
        rustValidForwardParty st m W →
          validForwardParty (poolViewOf st) st.canonical m W) ∧
        (∀ (st : RustValidForwardState) (led : SendRightsView) (mirrors : List Nat) (m W : Nat),
          rustValidMirrorMember st led mirrors m W →
            validMirror (poolViewOf st) st.canonical led m W) ∧
          validForwardPartyClosed

theorem valid_forward_refinement_closed : validForwardRefinementClosed :=
  ⟨fun st m W hde => rust_omit_de_whitelists st m W hde,
   fun st m e c W hW hpub hmiss =>
     rust_harvest_mismatch_de_whitelists st m e c W hW hpub hmiss,
   fun st m W h => rust_valid_forward_party_sound st m W h,
   fun st led mirrors m W h => rust_valid_mirror_set_subset st led mirrors m W h,
   valid_forward_party_closed⟩

end Refinement
