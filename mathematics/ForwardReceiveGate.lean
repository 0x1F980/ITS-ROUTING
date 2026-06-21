import ValidForwardParty
import ForwardProof
import WitnessConsensus

/-!
# Forward-receive gate — ValidFwd required before harvest (v9 ITS-A)

\[
\text{receiveGate}(m,e) \Leftrightarrow \text{ValidFwd}(m, [0, e-1])
\]

Harvest epoch `e` from mirror `m` requires valid forward history over prior epochs.
Alternate path uses only \(m \in \mathcal{M}_{\text{valid}}\) — no hop guilt.
-/

namespace ITS

/-- Forward history window for receive gate at epoch `e`: `[0, e-1]`. -/
def receiveGateWindow (e : Nat) : Nat :=
  e.pred

/-- ValidFwd over `[0, e-1]` required to harvest epoch `e` from mirror `m`. -/
def receiveGate (V : PoolView) (L : CanonicalLog) (m e : Nat) : Prop :=
  ∀ e', e' < e → ∀ c, published L e' c → V.harvest m e' = some c

theorem receive_gate_vacuous_at_zero
    (V : PoolView) (L : CanonicalLog) (m : Nat) :
    receiveGate V L m 0 :=
  fun e' he' _ _ => absurd he' (Nat.not_lt_zero e')

theorem valid_mirror_gives_receive_gate
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView) (m e W : Nat)
    (hvm : validMirror V L led m W)
    (he : e ≤ W + 1) :
    receiveGate V L m e := by
  intro e' he' c hpub
  have hle : e' ≤ W := by
    cases e with
    | zero => exact absurd he' (Nat.not_lt_zero e')
    | succ e₀ =>
      have he₀ : e' ≤ e₀ := Nat.le_of_lt_succ he'
      have he₀W : e₀ ≤ W := Nat.le_of_succ_le_succ he
      exact Nat.le_trans he₀ he₀W
  exact hvm.1 e' hle c hpub

theorem valid_mirror_set_member_gives_receive_gate
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (m e W : Nat)
    (hset : validMirrorSet V L led mirrors W)
    (hmem : m ∈ mirrors)
    (he : e ≤ W + 1) :
    receiveGate V L m e :=
  valid_mirror_gives_receive_gate V L led m e W (hset m hmem) he

/-- Harvest permitted at `e` when receive gate holds and mirror ∈ \(\mathcal{M}_{\text{valid}}\). -/
def harvestPermitted (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (m e c W : Nat) : Prop :=
  receiveGate V L m e →
    validMirror V L led m W →
      published L e c →
        V.harvest m e = some c

theorem receive_gate_permits_harvest
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView) (m e c W : Nat)
    (_hgate : receiveGate V L m e)
    (hvm : validMirror V L led m W)
    (hpub : published L e c)
    (he : e ≤ W) :
    V.harvest m e = some c :=
  hvm.1 e he c hpub

theorem harvest_permitted_of_gate
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView) (m e c W : Nat)
    (hgate : receiveGate V L m e)
    (hvm : validMirror V L led m W)
    (hpub : published L e c)
    (he : e ≤ W) :
    harvestPermitted V L led m e c W := by
  intro _ _ _
  exact receive_gate_permits_harvest V L led m e c W hgate hvm hpub he

/-- Alternate route via \(m \in \mathcal{M}_{\text{valid}}\) only — no hop guilt. -/
def alternateFromValidMirrors
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (s e c W : Nat) : Prop :=
  ∃ m ∈ mirrors,
    validMirror V L led m W ∧
      V.harvest s e ≠ some c ∧
        V.harvest m e = some c ∧
          published L e c

theorem alternate_from_valid_mirrors_gives_forward_proof
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (s e c W : Nat)
    (h : alternateFromValidMirrors V L led mirrors s e c W) :
    forwardProof V L e c := by
  obtain ⟨m, _, _, _, hharvest, hpub⟩ := h
  exact ⟨hpub, m, hharvest⟩

theorem alternate_from_valid_mirrors_gives_alternate_route
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (s m e c W : Nat)
    (_hmem : m ∈ mirrors)
    (_hvm : validMirror V L led m W)
    (hmiss : V.harvest s e ≠ some c)
    (hharvest : V.harvest m e = some c)
    (hpub : published L e c) :
    alternateRoute V L s m e c ∧
      forwardProof V L e c :=
  selective_omit_witness_gives_alternate_route V L s m e c hpub hmiss hharvest

/-- Membership in \(\mathcal{M}_{\text{valid}}\) + receive gate ⇒ canonical harvest. -/
theorem receive_gate_membership_canonical
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (m e c W : Nat)
    (hset : validMirrorSet V L led mirrors W)
    (hmem : m ∈ mirrors)
    (_hgate : receiveGate V L m e)
    (hpub : published L e c)
    (he : e ≤ W) :
    V.harvest m e = some c :=
  valid_mirror_harvest_matches V L led m e c W (hset m hmem) he hpub

/-- Closed bundle: alternate path from \(\mathcal{M}_{\text{valid}}\) ⇒ forward proof. -/
def forwardReceiveGateClosed : Prop :=
  ∀ (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (s e c W : Nat),
    alternateFromValidMirrors V L led mirrors s e c W →
      forwardProof V L e c

theorem forward_receive_gate_closed : forwardReceiveGateClosed :=
  fun V L led mirrors s e c W h =>
    alternate_from_valid_mirrors_gives_forward_proof V L led mirrors s e c W h

end ITS
