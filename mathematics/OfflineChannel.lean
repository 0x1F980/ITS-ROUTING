import Adversary
import ObservationAlphabet
import EndpointSplit
import IntegrityAxiom

/-!
# Offline channel — O_net = ∅ (sneakernet / total blackout)

I(S; O_net) = 0 trivially when network observation is empty.
Security reduces to wire algebra on offline medium + verify-oracle on Bob.
Availability is operational (A), not information-theoretic.
-/

namespace ITS

/-- Network observation empty (blackout / sneakernet). -/
def emptyNetworkObs : ChannelObs := { epochCells := 0 }

/-- O_net empty ⇒ zero channel mutual information. -/
def offlineChannelZero (secret : Nat) : Prop :=
  mutualInfo secret emptyNetworkObs.epochCells = 0

theorem offline_channel_zero (secret : Nat) : offlineChannelZero secret :=
  mutual_info_zero secret 0

/-- Offline delivery: same wire algebra; integrity on secure verify-oracle. -/
def offlineEndToEnd (ver : SecureVerifyOracle) : Prop :=
  offlineChannelZero 0 ∧ wireIntegrity ver

theorem offline_end_to_end (ver : SecureVerifyOracle) :
    offlineEndToEnd ver :=
  ⟨offline_channel_zero 0, fun ho hr => wire_integrity ver ho hr⟩

end ITS
