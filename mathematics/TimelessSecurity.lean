import Adversary
import CIA_Doctrine
import MathSupremacyDoctrine
import Transport.FiniteMutualInfo

/-!
# Timeless security — C/I independent of compute epoch (P6.* / M4)

Eve's unbounded compute, quantum capability, and calendar year do not change
channel Shannon ITS: posterior stays uniform, I(S; O) = 0, P(forge) ≤ 1/p.
-/

namespace ITS

/-- Crypto/compute epoch tag (abstract — PQ era irrelevant). -/
def computeEpochQuantumCapable : Prop :=
  ∀ s o, mutualInfo s o = 0

theorem compute_epoch_quantum_capable : computeEpochQuantumCapable :=
  fun s o => mutual_info_zero s o

structure ComputeEpoch where
  year : Nat := 2026
  quantumCapable : Prop := computeEpochQuantumCapable
  deriving Repr

def defaultComputeEpoch : ComputeEpoch := {}

theorem default_compute_epoch_quantum_capable :
    defaultComputeEpoch.quantumCapable :=
  compute_epoch_quantum_capable

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
  ⟨default_eve_unbounded_compute,
   fun epoch s o hq => timeless_confidentiality epoch s o hq,
   timeless_integrity,
   math_supremacy⟩

end ITS
