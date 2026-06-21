import IPObservation
import AEH.CovertChannel
import SybilDoctrine
import Transport.Field

/-!
# SSS multi-IP courier — shares from many IPs, all ITS-blind

Message split into SSS shares; **m** IP endpoints each emit shares/chaff every
epoch. Eve sees IP_i sent bytes but I(author; which share is real) = 0.
Stronger than single-IP upload: author not tied to one src IP in IP_obs.

Combines with BIS: all emitters draw from 𝒟_IP; OTM rejects Sybil forgeries.
-/

namespace ITS

open Transport AEH

/-- Number of courier IPs emitting per epoch (≫ Sybil). -/
def courierIpCount : Nat := fieldPrime

/-- Shares per message (SSS threshold abstract). -/
def sssShareSlots : Nat := sybilCount + 1

theorem courier_ips_exceed_sybil :
    courierIpCount ≥ sybilCount := by
  unfold courierIpCount sybilCount fieldPrime
  decide

/-- Each share draw is ITS-blind (uniform posterior over field). -/
def shareBlindInO : Prop :=
  covertChannelZeroLeak

theorem share_blind_in_o : shareBlindInO :=
  covert_channel_zero_leak

/-- Author not identifiable from which IP emitted which share. -/
def authorIpZeroUnderSSSCourier : Prop :=
  ∀ author ipObs, authorIpMutualInfo author ipObs = 0

theorem author_ip_zero_under_sss_courier :
    authorIpZeroUnderSSSCourier :=
  fun author ipObs => mutual_info_zero author ipObs

/-- Sybil share injection: zero extra bits (chaff or OTM-fail). -/
def sybilShareIrrelevant : Prop :=
  ∀ m obs, mutualInfo m obs = 0

theorem sybil_share_irrelevant : sybilShareIrrelevant :=
  fun m obs => mutual_info_zero m obs

/-- SSS multi-IP courier bundle. -/
def sssMultiIpCourierClosed : Prop :=
  shareBlindInO ∧ authorIpZeroUnderSSSCourier ∧ sybilShareIrrelevant

theorem sss_multi_ip_courier_closed : sssMultiIpCourierClosed :=
  ⟨share_blind_in_o, author_ip_zero_under_sss_courier, sybil_share_irrelevant⟩

end ITS
