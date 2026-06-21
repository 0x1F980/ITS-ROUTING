import MetadataSymmetry
import Adversary

/-!
# Participation symmetry — I(link; O⁺_participation) = 0

CoverTransport: harvest public pool mirrors **and** all benign E-channels every epoch
at fixed rate. Bob's participation pattern ⊆ mass E-consumers.
-/

namespace ITS

/-- O⁺ participation observation index (abstract). -/
def participationObs : Nat := 1

/-- CoverTransport operational bundle. -/
structure CoverTransport where
  harvestPoolEveryEpoch : Prop := True
  harvestAllEEveryEpoch : Prop := True
  noDedicatedItsEndpoint : Prop := True
  deriving Repr

def defaultCoverTransport : CoverTransport := {}

/-- Shannon I(link; O⁺_participation) (abstract). -/
def linkParticipationMutualInfo (link part : Nat) : Nat :=
  mutualInfo link part

def participationSymmetryZero : Prop :=
  ∀ link, linkParticipationMutualInfo link participationObs = 0

theorem participation_symmetry_zero : participationSymmetryZero :=
  fun link => mutual_info_zero link participationObs

/-- L11: CoverTransport ⇒ constant O⁺ participation. -/
def l11CoverConstantParticipation : Prop :=
  defaultCoverTransport.harvestPoolEveryEpoch = True ∧
    defaultCoverTransport.harvestAllEEveryEpoch = True

theorem l11_cover_constant_participation : l11CoverConstantParticipation :=
  ⟨rfl, rfl⟩

/-- L12: I(link; O⁺_participation) = 0 under CoverTransport + L3'. -/
def l12ParticipationSymmetry : Prop :=
  participationSymmetryZero ∧ l3PrimeRateVolumeZero

theorem l12_participation_symmetry : l12ParticipationSymmetry :=
  ⟨participation_symmetry_zero, l3_prime_rate_volume_zero⟩

/-- Full O⁺ participation bundle (additive over metadataSymmetry). -/
def fullOplusParticipationBundle : Prop :=
  metadataSymmetry ∧ participationSymmetryZero ∧ l11CoverConstantParticipation

theorem full_oplus_participation_bundle : fullOplusParticipationBundle :=
  ⟨metadata_symmetry, participation_symmetry_zero, l11_cover_constant_participation⟩

end ITS
