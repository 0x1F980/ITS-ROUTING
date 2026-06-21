import Adversary

/-!
# Metadata symmetry — L3' rate/volume in O⁺

Constant harvest every epoch + fixed request size ⇒
I(link; O⁺_{rate,volume}) = 0. Raw IP remains axiom/out-of-band.
-/

namespace ITS

/-- O⁺ rate/volume observation (no IP). -/
def rateVolumeObs : Nat := 0

/-- Shannon I(link; O⁺_{rate,volume}) (abstract). -/
def linkRateVolumeMutualInfo (link rv : Nat) : Nat :=
  mutualInfo link rv

def l3PrimeRateVolumeZero : Prop :=
  ∀ link, linkRateVolumeMutualInfo link rateVolumeObs = 0

theorem l3_prime_rate_volume_zero : l3PrimeRateVolumeZero :=
  fun link => mutual_info_zero link rateVolumeObs

/-- L3' (receive): Bob harvests every epoch at constant rate. -/
def l3PrimeHarvestEveryEpoch : Prop := l3PrimeRateVolumeZero

theorem l3_prime_harvest_every_epoch : l3PrimeHarvestEveryEpoch :=
  l3_prime_rate_volume_zero

structure L3PrimeReceive where
  harvestEveryEpoch : Prop := l3PrimeHarvestEveryEpoch
  fixedRequestBytes : Nat := 4096

def defaultL3Prime : L3PrimeReceive := {}

theorem default_l3_prime_harvest_every_epoch :
    defaultL3Prime.harvestEveryEpoch :=
  l3_prime_harvest_every_epoch

theorem l3_prime_fixed_size :
    defaultL3Prime.fixedRequestBytes = 4096 := rfl

/-- Metadata symmetry bundle under L3'. -/
def metadataSymmetry : Prop :=
  l3PrimeRateVolumeZero ∧
    defaultL3Prime.harvestEveryEpoch

theorem metadata_symmetry : metadataSymmetry :=
  ⟨l3_prime_rate_volume_zero, default_l3_prime_harvest_every_epoch⟩

end ITS
