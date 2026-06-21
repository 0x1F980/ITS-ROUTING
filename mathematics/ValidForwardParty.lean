import ForwardProof

/-!
# Valid forward party — math-driven mirror whitelist (v9 ITS-A)

\[
\text{ValidFwd}(m, W) \Leftrightarrow
  \forall e \leq W.\, \text{Publish}(e,c) \Rightarrow \text{Harvest}(m,e)=c
\]

Mirrors that fail selective omit are de-whitelisted (`¬validForwardParty`).
\(\mathcal{M}_{\text{valid}}\) = mirrors with ValidFwd and ¬sendRightsRevoked.
No hop guilt — harvester picks another \(m \in \mathcal{M}_{\text{valid}}\).

`SendRightsView` is AvailabilityLedger-compatible without importing it (avoids cycle).
-/

namespace ITS

/-- Ledger view — send-rights revocation predicate (AvailabilityLedger-compatible). -/
structure SendRightsView where
  revoked : Nat → Prop

/-- Mirror `m` correctly forwarded every published cell in window `[0, W]`. -/
def validForwardHistory (V : PoolView) (L : CanonicalLog) (m W : Nat) : Prop :=
  ∀ e, e ≤ W → ∀ c, published L e c → V.harvest m e = some c

/-- Valid-forward party: sustained correct harvest on the canonical log. -/
def validForwardParty (V : PoolView) (L : CanonicalLog) (m W : Nat) : Prop :=
  validForwardHistory V L m W

/-- Invalid forward party — de-whitelist candidate on omit. -/
def invalidForwardParty (V : PoolView) (L : CanonicalLog) (m W : Nat) : Prop :=
  ¬ validForwardParty V L m W

/-- Selective omit at epoch `e` breaks valid-forward history for mirror `m`. -/
theorem omit_de_whitelists_mirror
    (V : PoolView) (L : CanonicalLog) (m e c W : Nat)
    (hW : e ≤ W)
    (hpub : published L e c)
    (hmiss : V.harvest m e ≠ some c) :
    invalidForwardParty V L m W := by
  intro hvalid
  exact hmiss (hvalid e hW c hpub)

/-- Mirror `m` ∈ \(\mathcal{M}_{\text{valid}}\): ValidFwd and ledger send rights intact. -/
def validMirror (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (m W : Nat) : Prop :=
  validForwardParty V L m W ∧ ¬ led.revoked m

/-- Whitelist: every listed mirror is valid-forward and not slashed. -/
def validMirrorSet (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (W : Nat) : Prop :=
  ∀ m, m ∈ mirrors → validMirror V L led m W

theorem valid_mirror_harvest_matches
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView) (m e c W : Nat)
    (hvm : validMirror V L led m W)
    (hW : e ≤ W)
    (hpub : published L e c) :
    V.harvest m e = some c :=
  hvm.1 e hW c hpub

theorem valid_mirror_set_member_harvest
    (V : PoolView) (L : CanonicalLog) (led : SendRightsView)
    (mirrors : List Nat) (m e c W : Nat)
    (hset : validMirrorSet V L led mirrors W)
    (hmem : m ∈ mirrors)
    (hW : e ≤ W)
    (hpub : published L e c) :
    V.harvest m e = some c :=
  valid_mirror_harvest_matches V L led m e c W (hset m hmem) hW hpub

/-- Closed bundle: omit ⇒ de-whitelist. -/
def validForwardPartyClosed : Prop :=
  ∀ (V : PoolView) (L : CanonicalLog) (m e c W : Nat),
    e ≤ W → published L e c → V.harvest m e ≠ some c →
      invalidForwardParty V L m W

theorem valid_forward_party_closed : validForwardPartyClosed :=
  fun V L m e c W hW hpub hmiss =>
    omit_de_whitelists_mirror V L m e c W hW hpub hmiss

end ITS
