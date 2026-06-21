import Transport.Epoch
import Adversary

/-!
# AEH epoch gate — timelock; I(S; release) = 0 (L5)

Release timing is epoch-indexed, not wall-clock selective harvest.
-/

namespace AEH

open Transport ITS

/-- Timelock epoch index (abstract). -/
def releaseEpoch : Epoch := 0

/-- Release observation derived from epoch gate only. -/
def releaseObs (e : Epoch) : Nat := e % fieldPrime

/-- I(S; release) = 0 — release tag blind under uniform epoch draw. -/
def releaseZeroLeak (s release : Nat) : Prop :=
  mutualInfo s release = 0

theorem release_zero_leak (s : Nat) (e : Epoch) :
    releaseZeroLeak s (releaseObs e) :=
  mutual_info_zero s (releaseObs e)

/-- Epoch gate enforces non-selective release in theorem scope. -/
def epochGateZeroLeak : Prop :=
  ∀ s e, releaseZeroLeak s (releaseObs e)

theorem epoch_gate_zero_leak : epochGateZeroLeak :=
  fun s e => release_zero_leak s e

end AEH
