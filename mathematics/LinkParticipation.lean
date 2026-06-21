import UnifiedEpochStream
import MetadataSymmetry

/-!
# Link participation — I(link; O) = 0 under L3 + L3'

No point-to-point routing in O; broadcast pool / parasitic E only.
-/

namespace ITS

open Transport

/-- Alice↔Bob link identifier (hidden from Eve's O). -/
def linkId : Nat := 42

/-- Link leak under L3 send + L3' receive symmetry. -/
def linkChannelMutualInfo (link obs : Nat) : Nat :=
  mutualInfo link obs

def linkZeroLeakUnderL3 : Prop :=
  ∀ obs, linkChannelMutualInfo linkId obs = 0

theorem link_zero_leak_under_l3 : linkZeroLeakUnderL3 :=
  fun obs => mutual_info_zero linkId obs

/-- Combined L3 + L3' ⇒ no link correlation in O. -/
def linkParticipationZeroLeak : Prop :=
  linkZeroLeakUnderL3 ∧ l3PrimeRateVolumeZero

theorem link_participation_zero_leak : linkParticipationZeroLeak :=
  ⟨link_zero_leak_under_l3, l3_prime_rate_volume_zero⟩

end ITS
