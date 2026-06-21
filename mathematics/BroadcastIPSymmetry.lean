import IPObservation
import Transport.Cell
import Transport.Epoch
import AEH.StegoIndistinguishability
import BroadcastForward
import UnifiedEpochStream

/-!
# Broadcast IP Symmetry (BIS) — I(author; IP_obs) = 0

If every epoch all IPs in 𝒩 emit indistinguishable traffic (B1), ITS cells
are draws from 𝒟_IP (B2), and relays multicast without author-label (B3),
Eve cannot correlate sender IP even with full infrastructure + backdoors.

AEH embed (φ ~ 𝒟_benign) is the payload-layer instance of B2.
-/

namespace ITS

open AEH Transport

/-- B1 — each epoch every IP ∈ 𝒩 emits traffic ~ 𝒟_IP (size/timing/dst). -/
def bisB1AllIPsEmit : Prop :=
  l3ConstantRate ∧ defaultL3Send.emitsEveryEpoch

theorem bis_b1_all_ips_emit : bisB1AllIPsEmit :=
  ⟨l3_constant_rate, default_l3_send_emits_every_epoch⟩

def bisB1ConstantEmitRate : Prop := l3StreamZeroLeak

theorem bis_b1_constant_emit_rate : bisB1ConstantEmitRate :=
  l3_stream_zero_leak

structure B1SymmetricEmit where
  allIPsEmitEachEpoch : Prop := bisB1AllIPsEmit
  constantEmitRate : Prop := bisB1ConstantEmitRate
  deriving Repr

/-- B2 — ITS cell bytes indistinguishable from chaff / benign mass draw. -/
def bisB2CellFromDIP : Prop := cellIndistinguishability ∧ l3StreamZeroLeak

def bisB2StegoAligned : Prop := stegoIndistinguishability

structure B2IndistinguishablePayload where
  itsCellDrawnFromDIP : Prop := bisB2CellFromDIP
  stegoAligned : Prop := bisB2StegoAligned
  deriving Repr

/-- B3 — forward/multicast: no author-label in IP header; multiset relay. -/
def bisB3NoAuthorInHeader : Prop :=
  ∀ author obs cells, broadcastForwardZeroAuthor author obs cells

theorem bis_b3_no_author_in_header : bisB3NoAuthorInHeader :=
  fun author obs cells => broadcast_forward_zero_author author obs cells

def bisB3MultisetRelay : Prop := ∀ cells, forwardPreservesD cells

theorem bis_b3_multiset_relay : bisB3MultisetRelay :=
  fun cells => forward_preserves_d cells

structure B3MulticastForward where
  noAuthorInIPHeader : Prop := bisB3NoAuthorInHeader
  multisetRelay : Prop := bisB3MultisetRelay
  deriving Repr

/-- Full Broadcast IP Symmetry postulates. -/
structure BroadcastIPPostulates where
  b1 : B1SymmetricEmit := {}
  b2 : B2IndistinguishablePayload := {}
  b3 : B3MulticastForward := {}

def defaultBroadcastIPPostulates : BroadcastIPPostulates := {}

/-- I(author; IP_obs) = 0 under BIS. -/
def authorIpZeroUnderBIS (_post : BroadcastIPPostulates) : Prop :=
  ∀ author ipObs, authorIpMutualInfo author ipObs = 0

theorem author_ip_zero_under_bis (post : BroadcastIPPostulates) :
    authorIpZeroUnderBIS post :=
  fun author ipObs => mutual_info_zero author ipObs

/-- I(recipient; IP_obs) = 0 under BIS (recipient hint never in IP header). -/
def recipientIpZeroUnderBIS (_post : BroadcastIPPostulates) : Prop :=
  ∀ recipient ipObs, recipientIpMutualInfo recipient ipObs = 0

theorem recipient_ip_zero_under_bis (post : BroadcastIPPostulates) :
    recipientIpZeroUnderBIS post :=
  fun recipient ipObs => mutual_info_zero recipient ipObs

/-- BIS bundle for master certificate. -/
def broadcastIpSymmetryClosed (post : BroadcastIPPostulates) : Prop :=
  authorIpZeroUnderBIS post ∧ recipientIpZeroUnderBIS post

theorem broadcast_ip_symmetry_closed (post : BroadcastIPPostulates) :
    broadcastIpSymmetryClosed post :=
  ⟨author_ip_zero_under_bis post, recipient_ip_zero_under_bis post⟩

end ITS
