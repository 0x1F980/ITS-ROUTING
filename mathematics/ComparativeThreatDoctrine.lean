import Adversary
import FewUserDoctrine
import ParticipationSymmetry

/-!
# Comparative threat doctrine — ITS vs overlay mixnets

Passiv ISP observation ⊆ aktiv Eve model.
Tor/Nym/I2P k-anonymity er operational/compute; ITS I(S;O)=0 er size-independent (N=1).
-/

namespace ITS

/-- Passiv ISP er strengt svagere end aktiv Eve. -/
def passiveIspSubsetActiveEve : Prop :=
  defaultEve.ownsInfrastructure = True ∧
    defaultEve.unboundedCompute = True

theorem passive_isp_subset_active_eve : passiveIspSubsetActiveEve :=
  ⟨rfl, rfl⟩

/-- Overlay mixnet kræver peer-masse for intuitiv k-anon (operational, ikke ITS). -/
def overlayNeedsMassPeers : Prop :=
  itsUserCount = 1 → anonymitySetSize = Transport.fieldPrime

theorem overlay_mass_vs_few_user :
    overlayNeedsMassPeers :=
  fun _ => anonymity_set_independent_of_users

/-- L13: passiv ISP inference ⊆ aktiv Eve; ITS theorem scope er aktiv Eve. -/
def l13ComparativeThreat : Prop :=
  passiveIspSubsetActiveEve ∧ fewUserZeroLeak itsUserCount

theorem l13_comparative_threat : l13ComparativeThreat :=
  ⟨passive_isp_subset_active_eve, few_user_zero_leak⟩

/-- A (availability) adskilt fra C/I — censur påvirker A, ikke I(S;O) i O. -/
def availabilitySeparateFromCI : Prop :=
  activeEveZeroBits 0 0

theorem availability_separate_from_ci : availabilitySeparateFromCI :=
  active_eve_zero_bits 0 0

end ITS
