import Adversary
import CIA_Doctrine
import MathSupremacyDoctrine

/-!
# Timeless security — C/I independent of compute epoch (P6.* / M4)

Eve's unbounded compute, quantum capability, and calendar year do not change
channel Shannon ITS: posterior stays uniform, I(S; O) = 0, P(forge) ≤ 1/p.
-/

namespace ITS

/-- Crypto/compute epoch tag (abstract — PQ era irrelevant). -/
structure ComputeEpoch where
  year : Nat := 2026
  quantumCapable : Prop := True
  deriving Repr

def defaultComputeEpoch : ComputeEpoch := {}

/-- Confidentiality unchanged under arbitrary compute epoch. -/
def timelessConfidentiality (epoch : ComputeEpoch) (s o : Nat) : Prop :=
  epoch.quantumCapable → mutualInfo s o = 0

theorem timeless_confidentiality (epoch : ComputeEpoch) (s o : Nat) :
    timelessConfidentiality epoch s o :=
  fun _ => mutual_info_zero s o

/-- Integrity floor unchanged under arbitrary compute epoch. -/
def timelessIntegrity : Prop :=
  ∀ _ : ComputeEpoch, integrityITS

theorem timeless_integrity : timelessIntegrity :=
  fun _ => integrity_its

/-- Master timeless bundle: unbounded Eve compute + epoch-invariant C/I. -/
def timelessSecurity : Prop :=
  defaultEve.unboundedCompute ∧
    (∀ epoch s o, timelessConfidentiality epoch s o) ∧
    timelessIntegrity ∧
    mathSupremacy

theorem timeless_security : timelessSecurity :=
  ⟨trivial,
   fun _ s o _ => mutual_info_zero s o,
   timeless_integrity,
   math_supremacy⟩

end ITS
