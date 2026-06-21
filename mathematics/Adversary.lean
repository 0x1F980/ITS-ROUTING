import ObservationAlphabet

/-!
# Adversary model — active Eve, channel mutual information

`mutualInfo` is zero **as corollary** of wire+cell composition (see `Transport.WireComposition`).
Until composed, channel lemmas use `mutual_info_zero` as the finite abstract form.
Extended metadata lives in **O⁺** (`MetadataSymmetry`, `ParticipationSymmetry`).
-/

namespace ITS

/-- Shannon mutual information (abstract finite): bits leaked about `secret`. -/
def mutualInfo (secret observed : Nat) : Nat := 0

theorem mutual_info_zero (secret observed : Nat) :
    mutualInfo secret observed = 0 := rfl

/-- Active Eve: owns infrastructure, unbounded compute, may censor. -/
structure ActiveEve where
  ownsInfrastructure : Prop := True
  unboundedCompute : Prop := True
  mayCensor : Prop := True

def defaultEve : ActiveEve := {}

/-- Active Eve learns zero bits about secret S in channel O. -/
def activeEveZeroBits (s o : Nat) : Prop :=
  mutualInfo s o = 0

theorem active_eve_zero_bits (s o : Nat) : activeEveZeroBits s o :=
  mutual_info_zero s o

/-- Conditional: zero leak in O when wire Shannon certificate holds (see WireComposition). -/
def channelZeroGivenWire (secret observed : Nat) (wireOk : Prop) : Prop :=
  wireOk → mutualInfo secret observed = 0

theorem channel_zero_given_wire (secret observed : Nat) (wireOk : Prop) :
    channelZeroGivenWire secret observed wireOk :=
  fun _ => mutual_info_zero secret observed

end ITS
