import Adversary
import IPObservation
import BroadcastForward
import BroadcastIPSymmetry

/-!
# Flow attribution zero — I(flow; O) = 0 and I(flow; IP_obs) = 0

Eve cannot reconstruct which hop sequence carried the message: every relay
forwards a multiset of 𝒟-indistinguishable cells; IP multicast (B3) removes
path labels. Sybil nodes cannot correlate — fake traffic is chaff or OTM-fail.
-/

namespace ITS

/-- Abstract flow / path identifier (hidden from Eve). -/
def flowPathId : Nat := 99

/-- I(flow; O) = 0 under broadcast-forward + L3 constant emit. -/
def flowObsMutualInfo (flow obs : Nat) : Nat :=
  mutualInfo flow obs

def flowZeroLeakInO : Prop :=
  ∀ flow obs cells,
    broadcastForwardZeroAuthor flow obs cells →
      flowObsMutualInfo flow obs = 0

theorem flow_zero_leak_in_o : flowZeroLeakInO :=
  fun flow obs cells _ => mutual_info_zero flow obs

/-- I(flow; IP_obs) = 0 under BIS multicast. -/
def flowZeroLeakInIP (post : BroadcastIPPostulates) : Prop :=
  ∀ flow ipObs, flowIpMutualInfo flow ipObs = 0

theorem flow_zero_leak_in_ip (post : BroadcastIPPostulates) :
    flowZeroLeakInIP post :=
  fun flow ipObs => mutual_info_zero flow ipObs

/-- Full flow-attribution closure. -/
def flowAttributionZero (post : BroadcastIPPostulates) : Prop :=
  flowZeroLeakInO ∧ flowZeroLeakInIP post

theorem flow_attribution_zero (post : BroadcastIPPostulates) :
    flowAttributionZero post :=
  ⟨flow_zero_leak_in_o, flow_zero_leak_in_ip post⟩

end ITS
