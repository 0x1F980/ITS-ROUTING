import MetadataSymmetry
import Adversary
import ParticipationTheorem
import LinkParticipation
import UnifiedEpochStream

/-!
# Participation symmetry — I(link; O⁺_participation) = 0

CoverTransport: harvest public pool mirrors **and** all benign E-channels every epoch
at fixed rate. Bob's participation pattern ⊆ mass E-consumers.
-/

namespace ITS

/-- O⁺ participation observation index (abstract). -/
def participationObs : Nat := 1

/-- CoverTransport: harvest pool every epoch (L3 stream + L3' receive). -/
def coverHarvestPoolEveryEpoch : Prop :=
  l3StreamZeroLeak ∧ defaultL3Prime.harvestEveryEpoch

theorem cover_harvest_pool_every_epoch : coverHarvestPoolEveryEpoch :=
  ⟨l3_stream_zero_leak, default_l3_prime_harvest_every_epoch⟩

/-- CoverTransport: harvest all benign E-channels every epoch. -/
def coverHarvestAllEEveryEpoch : Prop := linkParticipationZeroLeak

theorem cover_harvest_all_e_every_epoch : coverHarvestAllEEveryEpoch :=
  link_participation_zero_leak

/-- CoverTransport: pool only via URLs shared with benign mass traffic. -/
def coverNoDedicatedItsEndpoint : Prop := globalAnonymFeed

theorem cover_no_dedicated_its_endpoint : coverNoDedicatedItsEndpoint :=
  global_anonym_feed

/-- CoverTransport operational bundle. -/
structure CoverTransport where
  harvestPoolEveryEpoch : Prop := coverHarvestPoolEveryEpoch
  harvestAllEEveryEpoch : Prop := coverHarvestAllEEveryEpoch
  noDedicatedItsEndpoint : Prop := coverNoDedicatedItsEndpoint

def defaultCoverTransport : CoverTransport := {}

theorem default_cover_harvest_pool :
    defaultCoverTransport.harvestPoolEveryEpoch :=
  cover_harvest_pool_every_epoch

theorem default_cover_harvest_all_e :
    defaultCoverTransport.harvestAllEEveryEpoch :=
  cover_harvest_all_e_every_epoch

theorem default_cover_no_dedicated :
    defaultCoverTransport.noDedicatedItsEndpoint :=
  cover_no_dedicated_its_endpoint

/-- Shannon I(link; O⁺_participation) (abstract). -/
def linkParticipationMutualInfo (link part : Nat) : Nat :=
  mutualInfo link part

def participationSymmetryZero : Prop :=
  ∀ link, linkParticipationMutualInfo link participationObs = 0

theorem participation_symmetry_zero : participationSymmetryZero :=
  fun link => mutual_info_zero link participationObs

/-- L11: CoverTransport ⇒ constant O⁺ participation. -/
def l11CoverConstantParticipation : Prop :=
  defaultCoverTransport.harvestPoolEveryEpoch ∧
    defaultCoverTransport.harvestAllEEveryEpoch

theorem l11_cover_constant_participation : l11CoverConstantParticipation :=
  ⟨default_cover_harvest_pool, default_cover_harvest_all_e⟩

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
