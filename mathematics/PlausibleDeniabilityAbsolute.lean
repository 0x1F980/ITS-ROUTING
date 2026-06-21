import AuthorAttributionZero
import RecipientAttributionZero
import FlowAttributionZero
import BroadcastIPSymmetry
import SSSMultiIPCourier
import EndpointEitherOr
import SybilDoctrine

/-!
# Absolute plausible deniability — master package (v4)

Eve owns 99.999%+ nodes; all pool/relay HW/SW is backdoored transcript.
Alice → Bob: **either** encryptor **or** verify-oracle is math-trusted.

**Theorem bundle:** Eve cannot correlate who sent, who received, or which
path carried the message — in **O** and **IP_obs** — under BIS + SSS courier.

Math drives security; software/hardware on Eve's side is irrelevant for C/I.
-/

namespace ITS

/-- Absolute deniability: sender + recipient + flow + IP + either-EP + Sybil. -/
def absolutePlausibleDeniability (post : BroadcastIPPostulates) : Prop :=
  authorAttributionZero ∧
    recipientAttributionZero post ∧
    flowAttributionZero post ∧
    broadcastIpSymmetryClosed post ∧
    sssMultiIpCourierClosed ∧
    eitherEndpointSecure defaultEncryptor defaultVerifyOracle ∧
    channelBlindUnderSybil ∧
    sybilIrrelevantForC

theorem absolute_plausible_deniability (post : BroadcastIPPostulates) :
    absolutePlausibleDeniability post :=
  ⟨author_attribution_zero,
   recipient_attribution_zero post,
   flow_attribution_zero post,
   broadcast_ip_symmetry_closed post,
   sss_multi_ip_courier_closed,
   either_endpoint_secure_default,
   channel_blind_under_sybil,
   sybil_irrelevant_for_c⟩

/-- Corollary: no guilty node in O or IP_obs — all plausibly deniable. -/
def noGuiltyNode (post : BroadcastIPPostulates) : Prop :=
  absolutePlausibleDeniability post ∧
    (∀ node obs, mutualInfo node obs = 0) ∧
    (∀ node ipObs, mutualInfo node ipObs = 0)

theorem no_guilty_node (post : BroadcastIPPostulates) :
    noGuiltyNode post :=
  ⟨absolute_plausible_deniability post,
   fun node obs => mutual_info_zero node obs,
   fun node ipObs => mutual_info_zero node ipObs⟩

end ITS
